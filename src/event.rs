use std::fmt::Display;
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
    MouseEvent,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct Events {
    pub output: tokio::sync::mpsc::UnboundedSender<Event>,
    pub input: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            output: tx,
            input: rx,
        }
    }

    pub fn send(&self, event: Event) -> Result<()> {
        Ok(self.output.send(event)?)
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.input
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Event stream closed"))
    }
}

#[derive(Debug)]
pub struct Tui {
    pub title: Option<String>,
    pub terminal: Terminal<CrosstermBackend<std::io::Stderr>>,
    pub task: Option<JoinHandle<()>>,
    pub events: Events,
    pub frame_rate: f64,
    pub timeout: f64,
}

impl Tui {
    pub fn start(&mut self) {
        let tick_delay = Duration::from_secs_f64(1.0 / self.timeout);
        let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);

        let _event_tx = self.events.output.clone();
        self.task = Some(tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    crossterm::event::Event::Key(mut key) => {
                                        if key.kind == KeyEventKind::Press {
                                            if let KeyCode::Char(value) = key.code {
                                                if !value.is_ascii_digit() && !value.is_alphabetic() {
                                                    key.modifiers &= !KeyModifiers::SHIFT;
                                                } else if value.is_ascii_uppercase() {
                                                    key.modifiers |= KeyModifiers::SHIFT;
                                                }
                                                _event_tx.send(Event::Key(KeyEvent::new(
                                                    KeyCode::Char(value.to_ascii_lowercase()),
                                                    key.modifiers
                                                ))).unwrap()
                                            } else {
                                                key.modifiers &= !KeyModifiers::SHIFT;
                                                _event_tx.send(Event::Key(KeyEvent::new(key.code, key.modifiers))).unwrap()
                                            }
                                        }
                                    },
                                    crossterm::event::Event::Mouse(mouse) => {
                                        _event_tx.send(Event::Mouse(mouse)).unwrap()
                                    }
                                    crossterm::event::Event::Resize(width, height) => {
                                        _event_tx.send(Event::Resize(width, height)).unwrap()
                                    }
                                    crossterm::event::Event::FocusGained => {
                                        _event_tx.send(Event::FocusGained).unwrap()
                                    }
                                    crossterm::event::Event::FocusLost => {
                                        _event_tx.send(Event::FocusLost).unwrap()
                                    }
                                    crossterm::event::Event::Paste(value) => {
                                        _event_tx.send(Event::Paste(value)).unwrap()
                                    }
                                }
                            },
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap()
                            },
                            None => {}
                        }
                    },
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap()
                    },
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap()
                    }
                }
            }
        }));
    }

    pub fn new() -> Result<Self> {
        Ok(Tui {
            title: None,
            terminal: Terminal::new(CrosstermBackend::new(std::io::stderr()))?,
            events: Events::new(),
            task: None,
            frame_rate: 30.0,
            timeout: 1.0,
        })
    }

    pub fn timeout(mut self, timeout: f64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn title<S: Display>(mut self, title: S) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(std::io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
        if let Some(title) = &self.title {
            execute!(std::io::stderr(), SetTitle(title))?;
        }

        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(info);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        self.start();
        Ok(())
    }

    pub fn reset() -> Result<()> {
        disable_raw_mode()?;
        execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        if let Some(task) = &self.task {
            task.abort();
        }
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> Result<()> {
        self.terminal.draw(f)?;
        Ok(())
    }
}
