use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyModifiers, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use crate::Error;

#[derive(Debug, Clone)]
pub enum Event {
    Tick(Duration),
    Render,
    Focus(bool),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64, render_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let render_rate = Duration::from_millis(render_rate);

        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut tick = tokio::time::interval(tick_rate);
            let mut render = tokio::time::interval(render_rate);

            let mut reader = crossterm::event::EventStream::new();
            loop {
                let tick_delay = tick.tick();
                let render_delay = render.tick();

                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    _sender.send(Event::Tick(tick_rate)).unwrap();
                  }
                  _ = render_delay => {
                    _sender.send(Event::Render).unwrap();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      CrosstermEvent::Key(mut key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            key.modifiers.remove(KeyModifiers::SHIFT);
                            _sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      CrosstermEvent::Mouse(mouse) => {
                        _sender.send(Event::Mouse(mouse)).unwrap();
                      },
                      CrosstermEvent::Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap();
                      },
                      CrosstermEvent::FocusLost => {
                        _sender.send(Event::Focus(false)).unwrap();
                      },
                      CrosstermEvent::FocusGained => {
                        _sender.send(Event::Focus(true)).unwrap();
                      },
                      CrosstermEvent::Paste(_) => {
                      },
                    }
                  }
                };
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> Result<Event, Error> {
        self.receiver
            .recv()
            .await
            .ok_or(Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to recieve next terminal event",
            )))
    }
}
