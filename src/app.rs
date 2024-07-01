use std::{collections::HashMap, io::stderr};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget,
    },
    Terminal,
};
use tokio::sync::mpsc;

use tupy::{
    api::{
        flow::{AuthFlow, Credentials, Pkce},
        request::Play,
        response::{Device, DeviceType, Item, Playback, PlaybackItem, Queue},
        scopes, validate_scope, OAuth, Spotify, UserApi,
    },
    DateTime, Duration, Error, Local,
};

use crate::{
    errors::install_hooks,
    spotify_util::listen_for_authentication_code,
    tui,
    ui::{centered_rect, playback::UiPlayback, window::UiQueue},
    Locked, Shared,
};

static FPS: u8 = 20;
lazy_static::lazy_static! {
    static ref FPS_DURATION: Duration = Duration::milliseconds((1.0 / FPS as f32) as i64 * 1000);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    Devices {
        state: ListState,
        devices: Vec<Device>,
    },
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Window {
    #[default]
    Home,
    Queue(ListState),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Viewport {
    Modal(Modal),
    #[default]
    Window,
}

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Focus,
    Unfocus,
    Tick,
    Close,
    Quit,
    None,

    Mouse(MouseEvent),

    Toggle,
    Next,
    Previous,

    Tab,
    PrevTab,
    Up,
    Down,
    Left,
    Right,
    Select,

    SelectDevice,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Countdown {
    count: usize,
    _origin: usize,
}
impl Countdown {
    pub fn new(origin: usize) -> Self {
        Self {
            count: origin,
            _origin: origin,
        }
    }

    pub fn decrement(&mut self) {
        self.count = self.count.saturating_sub(1);
    }
    pub fn poll(&mut self) -> bool {
        self.decrement();
        self.is_ready()
    }
    pub fn is_ready(&self) -> bool {
        self.count == 0
    }
    pub fn reset(&mut self) {
        self.count = self._origin;
    }
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub viewport: Viewport,
    pub window: Window,

    pub last_playback_poll: Shared<Locked<DateTime<Local>>>,
    pub playback_poll: Countdown,
    pub playback: Shared<Locked<Option<Playback>>>,

    pub queue: Shared<Locked<Option<Queue>>>,
}

#[derive(Debug)]
pub struct App {
    pub terminal: tui::Tui,
    pub focused: bool,
    pub quit: bool,

    pub spotify: Spotify<Pkce>,
    pub state: State,
}

impl App {
    pub async fn new() -> Result<Self> {
        let oauth = OAuth::from_env([
            scopes::USER_READ_PLAYBACK_STATE,
            scopes::USER_MODIFY_PLAYBACK_STATE,
            scopes::USER_READ_CURRENTLY_PLAYING,
        ])
        .expect("Failed to get TUPY_CLIENT_ID and TUPY_REDIRECT_URI environment variables.");

        let spotify =
            Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "rataify").unwrap();

        let app = Self {
            terminal: Terminal::new(CrosstermBackend::new(stderr())).unwrap(),
            focused: true,
            quit: false,

            spotify,
            state: State {
                playback_poll: Countdown::new(
                    FPS as usize * 3, /* Fetch playback every 3 seconds: Ticked once per frame (1/fps s) */
                ),
                ..Default::default()
            },
        };

        if validate_scope(
            app.spotify.api.scopes(),
            app.spotify.api.token().scopes.iter().map(|v| v.as_str()),
        )
        .is_err()
        {
            eprintln!("Failed to get Spotify token scopes, requesting new token");
            eprintln!("{:?}", app.spotify.api.scopes());
            eprintln!("{:?}", app.spotify.api.token().scopes);
            let auth_url = app.spotify.api.authorization_url(false)?;
            let auth_code = listen_for_authentication_code(
                &app.spotify.api.oauth.redirect,
                &auth_url,
                &app.spotify.api.oauth.state,
            )
            .await?;
            app.spotify.api.request_access_token(&auth_code).await?;
        }

        if app.spotify.api.token().is_expired() {
            app.spotify.api.refresh().await?;
        }

        *app.state.playback.lock().unwrap() = app.spotify.api.playback_state(None).await?;
        *app.state.last_playback_poll.lock().unwrap() = Local::now();

        Ok(app)
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            f.render_widget(self.state.clone(), f.size());
        })?;
        Ok(())
    }

    async fn update(&mut self, action: Action, tx: mpsc::UnboundedSender<Action>) -> Result<()> {
        match action {
            Action::Close => match self.state.viewport {
                Viewport::Modal(_) => {
                    self.state.viewport = Viewport::Window;
                }
                _ => self.quit = true,
            },
            Action::Quit => self.quit = true,
            Action::Focus if !self.focused => {
                self.focused = true;
            }
            Action::Unfocus if self.focused => {
                self.focused = false;
                // Restart the playback poll so that it starts over when focus is regained
                self.state.playback_poll.reset();
            }
            Action::Tick => {
                // Only render and poll for updates if the app is focused
                if self.focused {
                    self.render()?;

                    // Poll to check if playback state should be fetched
                    if self.state.playback_poll.poll() {
                        let playback = self.state.playback.clone();
                        let fetch_queue = if let Window::Queue(_) = self.state.window {
                            self.state.viewport == Viewport::Window
                        } else {
                            false
                        };
                        let queue = self.state.queue.clone();
                        let last_playback_poll = self.state.last_playback_poll.clone();
                        let api = self.spotify.api.clone();

                        tokio::task::spawn(async move {
                            // TODO: Push errors to error queue for displaying
                            // Also push it to a error log file that is cleared on startup
                            // TODO: Probably add a spot in the color_eyre panic hook to log errors
                            // and push them to the error queue
                            if api.token().is_expired() {
                                api.refresh().await.unwrap();
                            }

                            if let Ok(result) = api.playback_state(None).await {
                                *playback.lock().unwrap() = result;
                                *last_playback_poll.lock().unwrap() = Local::now();
                            }

                            if fetch_queue {
                                *queue.lock().unwrap() = Some(api.queue().await.unwrap());
                            }
                        });
                        self.state.playback_poll.reset();
                    }
                }
            }
            Action::Next => {
                if let Some(playback) = self.state.playback.lock().unwrap().as_ref() {
                    if let Some(device) = playback.device.as_ref() {
                        let api = self.spotify.api.clone();
                        let device = device.id.clone();
                        tokio::task::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.unwrap();
                            }
                            api.next(device).await.unwrap();
                        });
                    }
                }
            }
            Action::Previous => {
                if let Some(playback) = self.state.playback.lock().unwrap().as_ref() {
                    if let Some(device) = playback.device.as_ref() {
                        let api = self.spotify.api.clone();
                        let device = device.id.clone();
                        tokio::task::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.unwrap();
                            }
                            api.prev(device).await.unwrap();
                        });
                    }
                }
            }
            Action::Toggle => {
                let pb = self.state.playback.clone();
                if let Some(playback) = self.state.playback.lock().unwrap().as_mut() {
                    if playback.device.as_ref().is_some() {
                        let api = self.spotify.api.clone();
                        let poll = self.state.last_playback_poll.clone();
                        tokio::task::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.unwrap();
                            }

                            let playing = pb.lock().unwrap().as_ref().unwrap().is_playing;
                            if playing {
                                api.pause(None).await.unwrap();
                                if let Some(playback) = &mut (*pb.lock().unwrap()) {
                                    playback.is_playing = false;
                                    *poll.lock().unwrap() = Local::now();
                                }
                            } else {
                                api.play(Play::Resume, None).await.unwrap();
                                if let Some(playback) = &mut (*pb.lock().unwrap()) {
                                    playback.is_playing = true;
                                    *poll.lock().unwrap() = Local::now();
                                }
                            }
                        });
                        return Ok(());
                    }
                }

                tx.send(Action::SelectDevice)?;
            }
            Action::Down => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices { state, devices }) => {
                    next_in_list(state, devices.len());
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Home => {}
                    Window::Queue(state) => {
                        next_in_list(
                            state,
                            self.state
                                .queue
                                .lock()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .queue
                                .len(),
                        );
                    }
                },
            },
            Action::Up => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices { state, devices }) => {
                    prev_in_list(state, devices.len());
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Home => {}
                    Window::Queue(state) => {
                        prev_in_list(
                            state,
                            self.state
                                .queue
                                .lock()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .queue
                                .len(),
                        );
                    }
                },
            },
            Action::Select => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices { state, devices }) => {
                    let device = devices[state.selected().unwrap_or(0).saturating_sub(0)].clone();
                    let api = self.spotify.api.clone();
                    tokio::task::spawn(async move {
                        if api.token().is_expired() {
                            api.refresh().await.unwrap();
                        }
                        api.transfer_playback(device.id, true).await;
                    });
                    self.state.viewport = Viewport::Window;
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Home => {}
                    Window::Queue(state) => {
                        let queue = self.state.queue.clone();
                        let api = self.spotify.api.clone();
                        let pos = state.selected().unwrap_or(0);
                        tokio::task::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.unwrap();
                            }
                            let q = queue.lock().unwrap().clone();
                            if let Some(queue) = q {
                                api.play(
                                    Play::Queue {
                                        uris: queue
                                            .queue
                                            .iter()
                                            .map(|q| match q {
                                                Item::Track(t) => t.uri.clone(),
                                                Item::Episode(e) => e.uri.clone(),
                                            })
                                            .collect(),
                                        position: Duration::zero(),
                                        offset: Some(pos),
                                    },
                                    None,
                                )
                                .await
                                .unwrap();
                            }
                            *queue.lock().unwrap() = Some(api.queue().await.unwrap());
                        });
                        self.state.viewport = Viewport::Window;
                    }
                },
            },
            Action::SelectDevice => {
                if let Ok(devices) = self.spotify.api.devices().await {
                    self.state.viewport = Viewport::Modal(Modal::Devices {
                        state: ListState::default(),
                        devices,
                    });
                } else {
                    return Err(eyre!("Failed to get devices"));
                }
            }
            Action::Tab => match &self.state.window {
                Window::Home => {
                    if self.spotify.api.token().is_expired() {
                        self.spotify.api.refresh().await.unwrap();
                    }
                    self.state.window = Window::Queue(ListState::default());
                }
                Window::Queue(_) => {
                    self.state.window = Window::Home;
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_events(&self, keymaps: HashMap<KeyEvent, Action>, tx: mpsc::UnboundedSender<Action>) {
        let tick_rate = std::time::Duration::from_millis((1.0 / FPS as f32 * 1000.0) as u64);

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          crossterm::event::Event::Key(key) => {
                            if key.kind == crossterm::event::KeyEventKind::Press {
                              if let Some(action) = keymaps.get(&key) {
                                tx.send(*action).unwrap();
                              }
                            }
                          },
                          crossterm::event::Event::FocusGained => {
                            tx.send(Action::Focus).unwrap();
                          },
                          crossterm::event::Event::FocusLost => {
                            tx.send(Action::Unfocus).unwrap();
                          },
                          crossterm::event::Event::Mouse(mouse) => {
                            tx.send(Action::Mouse(mouse)).unwrap();
                          }
                          _ => {},
                        }
                      }
                      Some(Err(_)) => {
                        tx.send(Action::None).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = delay => {
                      tx.send(Action::Tick).unwrap();
                  },
                }
            }
        });
    }

    // Main Application Loop
    pub async fn run(&mut self, keymaps: HashMap<KeyEvent, Action>) -> Result<()> {
        install_hooks()?;

        tui::init()?;
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        self.handle_events(keymaps, action_tx.clone());

        while !self.quit {
            // application update
            if let Some(action) = action_rx.recv().await {
                self.update(action, action_tx.clone()).await?;
            }
        }

        tui::restore()?;
        Ok(())
    }
}

impl Widget for State {
    fn render(mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(5)])
            .split(area);

        // Viewport State Rendering
        match &mut self.viewport {
            Viewport::Modal(modal) => {
                modal.render(layout[0], buf);
            }
            Viewport::Window => match &mut self.window {
                Window::Home => {}
                Window::Queue(state) => {
                    UiQueue {
                        queue: self.queue.lock().unwrap().clone(),
                        state: state.clone(),
                    }
                    .render(layout[0], buf);
                }
            },
        }

        UiPlayback {
            playback: self.playback.clone(),
            last_playback_poll: self.last_playback_poll.clone(),
        }
        .render(layout[1], buf);
    }
}

fn next_in_list(list: &mut ListState, len: usize) {
    match list.selected() {
        Some(selected) if selected < len - 1 => {
            list.select(Some(selected + 1));
        }
        None => {
            list.select(Some(0));
        }
        _ => {}
    }
}

fn prev_in_list(list: &mut ListState, len: usize) {
    match list.selected() {
        Some(selected) if selected > 0 => {
            list.select(Some(selected - 1));
        }
        None => {
            list.select(Some(len - 1));
        }
        _ => {}
    }
}
