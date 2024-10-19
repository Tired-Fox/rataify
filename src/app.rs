use std::{io::stderr, time::Duration};

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hashbrown::HashMap;
use ratatui::{backend::CrosstermBackend, Frame};
use rspotify::{clients::OAuthClient, model::{parse_uri, Id, PlayContextId}};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    action::{Action, ModalOpen, Play},
    event::Event,
    state::{window::modal::Modal, InnerState, State},
    Error, ErrorKind, Tui,
};

#[derive(Clone)]
pub struct ContextSender {
    pub action_sender: mpsc::UnboundedSender<Action>,
    pub error_sender: mpsc::UnboundedSender<Error>,
}

impl ContextSender {
    pub fn new(
        action_sender: mpsc::UnboundedSender<Action>,
        error_sender: mpsc::UnboundedSender<Error>,
    ) -> Self {
        Self {
            action_sender,
            error_sender,
        }
    }

    pub fn send_action(&self, action: Action) -> Result<(), Error> {
        Ok(self.action_sender.send(action)?)
    }

    pub fn send_error(&self, error: Error) -> Result<(), Error> {
        Ok(self.error_sender.send(error)?)
    }
}

pub struct ContextReceiver {
    pub action_receiver: mpsc::UnboundedReceiver<Action>,
    pub error_receiver: mpsc::UnboundedReceiver<Error>,
}

impl ContextReceiver {
    pub fn new(
        action_receiver: mpsc::UnboundedReceiver<Action>,
        error_receiver: mpsc::UnboundedReceiver<Error>,
    ) -> Self {
        Self {
            action_receiver,
            error_receiver,
        }
    }

    pub fn try_next_action(&mut self) -> Result<Option<Action>, Error> {
        match self.action_receiver.try_recv() {
            Err(TryRecvError::Empty) => Ok(None),
            Err(other) => Err(Error::from(other)),
            Ok(v) => Ok(Some(v)),
        }
    }

    pub fn try_next_error(&mut self) -> Result<Option<Error>, Error> {
        match self.error_receiver.try_recv() {
            Err(TryRecvError::Empty) => Ok(None),
            Err(other) => Err(Error::from(other)),
            Ok(v) => Ok(Some(v)),
        }
    }
}

pub struct App {
    quit: bool,
    keymaps: HashMap<KeyEvent, Action>,

    context_sender: ContextSender,
    context_receiver: ContextReceiver,

    state: State,
}

impl App {
    pub async fn new(keymaps: impl Into<HashMap<KeyEvent, Action>>) -> Result<Self, Error> {
        let (action_sender, action_receiver) = mpsc::unbounded_channel();
        let (error_sender, error_receiver) = mpsc::unbounded_channel();

        let context_sender = ContextSender::new(action_sender, error_sender);
        let context_receiver = ContextReceiver::new(action_receiver, error_receiver);
        Ok(Self {
            quit: false,
            keymaps: keymaps.into(),

            state: State::new(InnerState::default(), context_sender.clone()).await?,

            context_sender,
            context_receiver,
        })
    }

    async fn tick(&mut self, dt: Duration) -> Result<(), Error> {
        self.state.tick(dt).await
    }

    async fn open_device_modal(&mut self, play: Option<bool>) -> Result<(), Error> {
        let devices = self.state.spotify.device().await?;
        self.state
            .inner
            .devices
            .lock()
            .unwrap()
            .reset(devices, play);
        self.state
            .inner
            .modal
            .lock()
            .unwrap()
            .replace(Modal::Devices);
        Ok(())
    }

    async fn update(&mut self) -> Result<(), Error> {
        while let Ok(Some(action)) = self.context_receiver.try_next_action() {
            match action {
                Action::Quit => self.quit = true,
                Action::Close => {
                    self.quit = self.state.close_modal();
                }
                Action::Next => {
                    let spotify = self.state.spotify.clone();
                    let ctx = self.context_sender.clone();
                    tokio::spawn(async move {
                        if spotify.next_track(None).await.is_err() {
                            ctx.send_action(Action::Open(ModalOpen::devices(Some(true))))
                                .unwrap();
                        }
                    });
                }
                Action::Previous => {
                    let spotify = self.state.spotify.clone();
                    let ctx = self.context_sender.clone();
                    tokio::spawn(async move {
                        if spotify.previous_track(None).await.is_err() {
                            ctx.send_action(Action::Open(ModalOpen::devices(Some(true))))
                                .unwrap();
                        }
                    });
                }
                Action::Toggle => {
                    let spotify = self.state.spotify.clone();
                    let playback = self.state.inner.playback.clone();
                    let ctx = self.context_sender.clone();
                    tokio::spawn(async move {
                        let playing = playback
                            .lock()
                            .unwrap()
                            .as_ref()
                            .map(|p| p.is_playing)
                            .unwrap_or_default();
                        if playing {
                            if spotify.pause_playback(None).await.is_err() {
                                ctx.send_action(Action::Open(ModalOpen::devices(Some(false))))
                                    .unwrap();
                                return;
                            }

                            if let Some(playback) = playback.lock().unwrap().as_mut() {
                                playback.is_playing = false;
                                playback.timestamp = Local::now();
                            }
                        } else {
                            if spotify.resume_playback(None, None).await.is_err() {
                                ctx.send_action(Action::Open(ModalOpen::devices(Some(true))))
                                    .unwrap();
                                return;
                            }

                            if let Some(playback) = playback.lock().unwrap().as_mut() {
                                playback.is_playing = true;
                            }
                        }
                    });
                }
                Action::Play(play) => match play {
                    Play::Context(id, offset, position) => {
                        let spotify = self.state.spotify.clone();
                        let ctx = self.context_sender.clone();
                        tokio::spawn(async move {
                            if spotify
                                .start_context_playback(id.play_context_id().unwrap(), None, offset.map(|v| v.into()), position.map(|v| chrono::Duration::milliseconds(v as i64)))
                                .await
                                .is_err()
                            {
                                ctx.send_action(Action::Open(ModalOpen::devices(Some(true))))
                                    .unwrap();
                            }
                        });
                    }
                },
                Action::Open(modal) => match modal {
                    ModalOpen::Devices(play) => self.open_device_modal(play).await?,
                },
                Action::SetDevice(id, play) => {
                    self.state
                        .spotify
                        .transfer_playback(id.as_str(), play)
                        .await?;

                    let spot = self.state.spotify.clone();
                    let playback = self.state.inner.playback.clone();
                    tokio::spawn(async move {
                        let ctx = spot
                            .current_playback(None, None::<Vec<_>>)
                            .await
                            .unwrap()
                            .map(|v| v.into());
                        *playback.lock().unwrap() = ctx;
                    });
                }
                other => self
                    .state
                    .handle_action(other, self.context_sender.clone())?,
            }
        }

        let mut errors = Vec::new();
        while let Ok(Some(error)) = self.context_receiver.try_next_error() {
            errors.push(error)
        }
        if !errors.is_empty() {
            return Err(Error {
                kind: ErrorKind::Group(errors),
            });
        }

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut self.state, frame.area());
    }

    fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Key(ke) => match ke {
                KeyEvent {
                    code: KeyCode::Char('c' | 'C'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.context_sender.send_action(Action::Quit)?,
                other => match self.keymaps.get(&other) {
                    Some(action) => self.context_sender.send_action(action.clone())?,
                    None => self.context_sender.send_action(Action::Key(other))?,
                },
            },
            Event::Mouse(_me) => {}
            Event::Focus(_focus) => {}
            Event::Resize(_w, _h) => {}
            _ => {}
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let mut tui = Tui::new(CrosstermBackend::new(stderr()), 60)?;
        tui.init()?;

        while !self.quit {
            match tui.events.next().await? {
                Event::Tick(dt) => {
                    self.tick(dt).await?;
                    self.update().await?;

                    tui.draw(|f| self.render(f))?;
                }
                other => self.handle_event(other)?,
            }
        }

        tui.exit()?;
        Ok(())
    }
}
