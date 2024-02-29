use std::collections::HashMap;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use crate::action;
use crate::action::{Action, Private, Public};
use crate::config::Config;
use crate::error::Error;
use crate::event::{Event, Tui};
use crate::spotify::body::StartPlayback;
use crate::spotify::Spotify;
use crate::state::State;

pub struct App {
    /// App should quit on next event loop
    should_quit: bool,
    ui: Option<Box<dyn FnMut(&mut State, &mut Frame) + 'static>>,
    /// Actions Output Channel
    pub actions: tokio::sync::mpsc::UnboundedSender<Action>,
    pub spotify: Spotify,

    /// Actions Input Channel
    input: tokio::sync::mpsc::UnboundedReceiver<Action>,
    playback_handle: tokio::task::JoinHandle<()>,

    /// State
    state: State,
}

impl App {
    /// Async app setup to also initialize the spotify api interactions.
    ///
    /// The interactions require an access token so http requests may run on init.
    pub async fn new() -> color_eyre::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let actions = tx.clone();

        // Start playback update loop, each loop interval will send a playback update action
        // this action will then be handled by the apps update loop
        let playback = tokio::task::spawn(async move {
            loop {
                actions.send(Action::Private(Private::UpdatePlayback)).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        Ok(Self {
            should_quit: false,
            ui: None,
            actions: tx,
            spotify: Spotify::new().await.unwrap(),

            input: rx,
            playback_handle: playback,
            state: State::new().await,
        })
    }

    pub fn with_ui<F>(mut self, ui: F) -> Self
        where
            F: FnMut(&mut State, &mut Frame) + 'static + Clone
    {
        self.ui = Some(Box::new(ui));
        self
    }

    async fn update(&mut self, action: Action) -> color_eyre::Result<()> {
        // TODO: Handle errors into actions for user feedback
        macro_rules! call_with_device {
            ($call: expr, $action: expr) => {
                 match $call.await {
                     Err(Error::NoDevice) => {
                         self.actions.send(Action::Public(Public::SelectDevice))?;
                         self.actions.send($action)?;
                     }
                     val => val?
                 }
            };
        }

        match action {
            Action::Public(public) => match public {
                Public::Next => call_with_device!(self.spotify.next(), action::public!(Next)),
                Public::Previous => call_with_device!(self.spotify.previous(), action::public!(Previous)),
                Public::Play => call_with_device!(self.spotify.play(&StartPlayback::default()), action::public!(Play)),
                Public::Pause => call_with_device!(self.spotify.pause(), action::public!(Pause)),
                Public::TogglePlayback => match self.state.playback.playing() {
                    true => call_with_device!(self.spotify.pause(), action::public!(Pause)),
                    false => call_with_device!(self.spotify.play(&StartPlayback::default()), action::public!(Play)),
                }
                Public::Close => {}
                Public::Exit => self.should_quit = true,
                Public::SelectDevice => {
                    todo!("Set window state to select device for modal")
                }
                _ => unimplemented!()
            }
            Action::Private(private) => match private {
                Private::Tick => {}
                Private::UpdatePlayback => self.state.playback.now_playing(self.spotify.playback().await.ok()),
                _ => {}
            }
            _ => {}
        }

        Ok(())
    }

    fn get_action(&mut self, event: Event, keymap: &HashMap<KeyEvent, Action>) -> Action {
        match event {
            Event::Quit => Action::from(Public::Exit),
            Event::Error => Action::None,
            Event::Tick => Action::from(Private::Tick),
            Event::Render => Action::from(Private::Render),
            Event::Key(key) => {
                if keymap.contains_key(&key) {
                    return *keymap.get(&key).unwrap();
                }
                Action::None
            }
            _ => Action::None
        }
    }

    pub async fn run(&mut self, config: Config) -> color_eyre::Result<()> {
        let keymaps = &config.keymaps;

        let mut terminal = Tui::new()?.title("Rataify");
        terminal.enter()?;

        loop {
            let event = terminal.events.next().await?;

            let action = self.get_action(event, keymaps);
            self.actions.send(action.clone())?;

            while let Ok(action) = self.input.try_recv() {
                if let Action::Private(Private::Render) = action {
                    if let Some(ui) = &mut self.ui {
                        terminal.draw(|frame: &mut Frame| ui(&mut self.state, frame)).unwrap();
                    }
                } else {
                    self.update(action).await?;
                }
            }

            if self.should_quit {
                break;
            }
        }

        terminal.exit()?;
        Ok(())
    }
}