use std::{io::stderr, time::Duration};

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hashbrown::HashMap;
use ratatui::{backend::CrosstermBackend, Frame};
use rspotify::{clients::OAuthClient, model::PlayContextId};
use tokio::sync::mpsc;

use crate::{action::{Action, Play}, event::Event, state::{InnerState, State}, Error, Tui};

pub struct App {
    quit: bool,
    keymaps: HashMap<KeyEvent, Action>,

    sender: mpsc::UnboundedSender<Action>,
    reciever: mpsc::UnboundedReceiver<Action>,

    state: State
}

impl App {
    pub async fn new(keymaps: impl Into<HashMap<KeyEvent, Action>>) -> Result<Self, Error> {
        let (sender, reciever) = mpsc::unbounded_channel();

        Ok(Self {
            quit: false,
            keymaps: keymaps.into(),

            sender,
            reciever,

            state: State::new(InnerState::default()).await?
        })
    }

    async fn tick(&mut self, dt: Duration) -> Result<(), Error> {
        self.state.tick(dt).await
    }

    fn update(&mut self) -> Result<(), Error> {
        while let Ok(action) = self.reciever.try_recv() {
            match action {
                Action::Quit => self.quit = true,
                Action::Close => {
                    self.quit = self.state.close_modal();
                },
                Action::Next => {
                    let spotify = self.state.spotify.clone();
                    tokio::spawn(async move {
                        spotify.next_track(None).await.unwrap();
                    });
                }
                Action::Previous => {
                    let spotify = self.state.spotify.clone();
                    tokio::spawn(async move {
                        spotify.previous_track(None).await.unwrap();
                    });
                }
                Action::Toggle => {
                    let spotify = self.state.spotify.clone();
                    let playback = self.state.inner.playback.clone();
                    tokio::spawn(async move {
                        let playing = playback.lock().unwrap().as_ref().map(|p| p.is_playing).unwrap_or_default();
                        if playing {
                            spotify.pause_playback(None).await.unwrap();
                            if let Some(playback) = playback.lock().unwrap().as_mut() {
                                playback.is_playing = false;
                                playback.timestamp = Local::now();
                            }
                        } else {
                            spotify.resume_playback(None, None).await.unwrap();
                            if let Some(playback) = playback.lock().unwrap().as_mut() {
                                playback.is_playing = true;
                            }
                        }
                    });
                }
                Action::Play(play) => match play {
                    Play::Context(id, offset, position) => {
                        let spotify = self.state.spotify.clone();
                        tokio::spawn(async move {
                            spotify.start_context_playback(id, None, offset, position).await.unwrap();
                        });
                    }
                }
                other => {
                    self.state.handle_action(other, self.sender.clone())?
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut self.state, frame.area());
    }

    fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Key(ke) => {
                match ke {
                    KeyEvent { code: KeyCode::Char('c' | 'C'), modifiers: KeyModifiers::CONTROL, .. } => self.sender.send(Action::Quit)?,
                    other => {
                        match self.keymaps.get(&other) {
                            Some(action) => self.sender.send(action.clone())?,
                            None => self.sender.send(Action::Key(other))?
                        }
                    }
                }
            },
            Event::Mouse(_me) => {},
            Event::Focus(_focus) => {},
            Event::Resize(_w, _h) => {},
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
                    self.update()?;

                    tui.draw(|f| {
                        self.render(f)
                    })?;
                },
                other => self.handle_event(other)?,
            }
        }

        tui.exit()?;
        Ok(())
    }
}
