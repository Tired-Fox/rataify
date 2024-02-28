use std::collections::HashMap;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use crate::action::{Action, Private, Public};
use crate::config::Config;
use crate::event::{Event, Tui};
use crate::state::State;

pub struct App {
    /// App should quit on next event loop
    should_quit: bool,
    ui: Option<Box<dyn FnMut(&State, &mut Frame) + 'static>>,
    /// Actions Output Channel
    pub actions: tokio::sync::mpsc::UnboundedSender<Action>,
    /// Actions Input Channel
    input: tokio::sync::mpsc::UnboundedReceiver<Action>,

    /// State
    state: State,
}

impl App {
    /// Async app setup to also initialize the spotify api interactions.
    ///
    /// The interactions require an access token so http requests may run on init.
    pub async fn new() -> crate::error::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Ok(Self {
            should_quit: false,
            ui: None,
            actions: tx,
            input: rx,
            state: State::new().await,
        })
    }

    pub fn with_ui<F>(mut self, ui: F) -> Self
        where
            F: FnMut(&State, &mut Frame) + 'static + Clone
    {
        self.ui = Some(Box::new(ui));
        self
    }

    async fn update(&mut self, action: Action) {
        match action {
            Action::Public(public) => match public {
                Public::Next => self.state.next().await,
                Public::Previous => self.state.previous().await,
                Public::Close => {}
                Public::Exit => self.should_quit = true,
                _ => unimplemented!()
            }
            Action::Private(private) => match private {
                Private::Tick => {}
                _ => {}
            }
            _ => {}
        }
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

    pub async fn run(&mut self, config: Config) -> crate::error::Result<()> {
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
                        terminal.draw(|frame: &mut Frame| ui(&self.state, frame)).unwrap();
                    }
                } else {
                    self.update(action).await;
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