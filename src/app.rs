use std::{io::stderr, time::Duration};

use crossterm::event::KeyEvent;
use hashbrown::HashMap;
use ratatui::{backend::CrosstermBackend, Frame};
use tokio::sync::mpsc;

use crate::{action::Action, event::Event, state::{InnerState, State}, Error, Tui};

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
                Action::Close => self.quit = true,
            }
        }

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut self.state, frame.area());
    }

    fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Key(ke) => if let Some(action) = self.keymaps.get(&ke) {
                self.sender.send(*action).unwrap();
            }
            Event::Mouse(_me) => {},
            Event::Focus(_focus) => {},
            Event::Resize(_w, _h) => {},
            _ => {}
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let mut tui = Tui::new(CrosstermBackend::new(stderr()), 250, 33)?;
        tui.init()?;

        while !self.quit {
            match tui.events.next().await? {
                Event::Tick(dt) => self.tick(dt).await?,
                Event::Render => {
                    tui.draw(|f| {
                        self.render(f)
                    })?;
                },
                other => self.handle_event(other)?,
            }

            self.update()?;
        }

        tui.exit()?;
        Ok(())
    }
}
