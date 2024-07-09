use std::{collections::HashMap, io::stderr};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend,
    widgets::TableState,
    Terminal,
};
use tokio::sync::mpsc;

use tupy::{
    api::{
        flow::{AuthFlow, Credentials, Pkce}, request::Play, response::{Item, PlaybackAction, PlaybackItem, Repeat}, scopes, OAuth, Resource, Spotify, Uri, UserApi
    },
    Local,
};

use crate::{
    errors::install_hooks, key, spotify_util::listen_for_authentication_code, state::{modal::{ActionState, DevicesState}, playback::Playback, window::{landing::Landing, queue::Queue}, Countdown, Modal, State, Viewport, Window}, tui, ui::{self, GoTo, IntoActions}
};

static FPS: usize = 24;

#[derive(Debug, Clone)]
pub enum Event {
    // Implicit internal actions
    Focus,
    Unfocus,
    Tick,
    None,
    GoTo(GoTo),

    UpdateQueue,

    // Close/Quit
    Close,
    Quit,

    // Playback
    Toggle,
    Next,
    Previous,
    Play(Play),
    ToggleRepeat,
    ToggleShuffle,
    VolumeUp,
    VolumeDown,

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Select,
    Tab,
    Backtab,

    // Open menu
    OpenAddToPlaylist(Uri),
    OpenSelectDevice,
    OpenGoTo,
    OpenAction,
    OpenHelp,
    OpenSearch,

    // Misc input events
    Key(KeyEvent),
    Mouse(MouseEvent),
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
            scopes::USER_LIBRARY_READ,
            scopes::USER_LIBRARY_MODIFY,
            scopes::USER_FOLLOW_READ,
            scopes::USER_READ_CURRENTLY_PLAYING,
            scopes::USER_READ_PLAYBACK_STATE,
            scopes::USER_MODIFY_PLAYBACK_STATE,
            scopes::USER_READ_PLAYBACK_POSITION,
            scopes::PLAYLIST_READ_PRIVATE,
            scopes::PLAYLIST_MODIFY_PUBLIC,
        ])
        .expect("Failed to get TUPY_CLIENT_ID and TUPY_REDIRECT_URI environment variables.");

        let spotify =
            Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "rataify").unwrap();

        if spotify.api.scopes() != &spotify.api.token().scopes {
            eprintln!("Failed to get Spotify token scopes, requesting new token");
            eprintln!("{:?}", spotify.api.scopes());
            eprintln!("{:?}", spotify.api.token().scopes);
            let auth_url = spotify.api.authorization_url(false)?;
            let auth_code = listen_for_authentication_code(
                &spotify.api.oauth.redirect,
                &auth_url,
                &spotify.api.oauth.state,
            )
            .await?;
            spotify.api.request_access_token(&auth_code).await?;
        }

        if spotify.api.token().is_expired() {
            spotify.api.refresh().await?;
        }
        
        let mut playback = spotify.api.playback_state(None).await?.map(|pb| Playback::from(pb));
        if let Some(playback) = playback.as_mut() {
            playback.saved = match &playback.item {
                PlaybackItem::Track(t) => spotify.api.check_saved_tracks([t.uri.clone()]).await.unwrap()[0],
                PlaybackItem::Episode(e) => spotify.api.check_saved_episodes([e.uri.clone()]).await.unwrap()[0],
                _ => false,
            };
        }

        let app = Self {
            terminal: Terminal::new(CrosstermBackend::new(stderr())).unwrap(),
            focused: true,
            quit: false,

            state: State::new(
                "rataify",
                &spotify.api,
                Countdown::new(FPS * 3),
                playback        
            ).await?,
            spotify,
        };

        Ok(app)
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            f.render_widget(self.state.clone(), f.size());
        })?;
        Ok(())
    }

    async fn update(&mut self, action: Event, keymaps: &HashMap<KeyEvent, Event>, tx: mpsc::UnboundedSender<Event>) -> Result<()> {
        match action {
            Event::Close => match self.state.viewport {
                Viewport::Modal(_) => {
                    self.state.viewport = Viewport::Window;
                }
                _ => self.quit = true,
            },
            Event::Quit => self.quit = true,
            Event::Focus if !self.focused => {
                self.focused = true;
            }
            Event::Unfocus if self.focused => {
                self.focused = false;
                // Restart the playback poll so that it starts over when focus is regained
                self.state.playback_poll.reset();
            }
            Event::Tick => {
                // Only render and poll for updates if the app is focused
                self.render()?;

                // Poll to check if playback state should be fetched
                if self.state.playback_poll.poll() {
                    let playback = self.state.playback.clone();
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
                            let diff = playback.lock().unwrap().set_playback(result.map(|pb| pb.into()));
                            if diff && playback.lock().unwrap().is_some() {
                                let item = playback.lock().unwrap().playback.as_ref().unwrap().item.clone();
                                let saved = match &item {
                                    PlaybackItem::Track(t) => api.check_saved_tracks([t.uri.clone()]).await.unwrap()[0],
                                    PlaybackItem::Episode(e) => api.check_saved_episodes([e.uri.clone()]).await.unwrap()[0],
                                    _ => false,
                                };
                                playback.lock().unwrap().set_saved(saved);
                            }

                            tx.send(Event::UpdateQueue).unwrap();
                        }
                    });
                    self.state.playback_poll.reset();
                }
            }
            Event::Next => {
                if let Some(playback) = self.state.playback.lock().unwrap().playback.as_ref() {
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
            Event::Previous => {
                if let Some(playback) = self.state.playback.lock().unwrap().playback.as_ref() {
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
            Event::Toggle => {
                if let Some(playback) = self.state.playback.lock().unwrap().playback.as_mut() {
                    if playback.device.as_ref().is_some() {
                        let api = self.spotify.api.clone();
                        let pb = self.state.playback.clone();
                        tokio::task::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.unwrap();
                            }

                            let playing = pb.lock().unwrap().playback.as_ref().unwrap().is_playing;
                            if playing {
                                api.pause(None).await.unwrap();
                                if let Some(playback) = &mut (*pb.lock().unwrap()).playback {
                                    playback.is_playing = false;
                                    pb.lock().unwrap().last_playback_poll = Local::now();
                                }
                            } else {
                                api.play(Play::Resume, None).await.unwrap();
                                if let Some(playback) = &mut (*pb.lock().unwrap()).playback {
                                    playback.is_playing = true;
                                    pb.lock().unwrap().last_playback_poll = Local::now();
                                }
                            }
                        });
                        return Ok(());
                    }
                }

                tx.send(Event::OpenSelectDevice)?;
            }
            Event::Down => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices) => {
                    self.state.modal_state.devices.lock().unwrap().next()
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Queue => self.state.window_state.queue.lock().unwrap().next(),
                    Window::Library => self.state.window_state.library.lock().unwrap().down().await,
                    Window::Landing => self.state.window_state.landing.lock().unwrap().down(),
                    _ => {}
                },
                _ => {}
            },
            Event::Up => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices) => {
                    self.state.modal_state.devices.lock().unwrap().prev()
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Queue => self.state.window_state.queue.lock().unwrap().prev(),
                    Window::Library => self.state.window_state.library.lock().unwrap().up().await,
                    Window::Landing => self.state.window_state.landing.lock().unwrap().up(),
                    _ => {}
                },
                _ => {}
            },
            Event::Right => match &mut self.state.viewport {
                Viewport::Window => match &mut self.state.window {
                    Window::Library => self.state.window_state.library.lock().unwrap().right().await?,
                    Window::Landing => self.state.window_state.landing.lock().unwrap().right().await?,
                    _ => {}
                },
                _ => {}
            },
            Event::Left => match &mut self.state.viewport {
                Viewport::Window => match &mut self.state.window {
                    Window::Library => self.state.window_state.library.lock().unwrap().left().await?,
                    Window::Landing => self.state.window_state.landing.lock().unwrap().left().await?,
                    _ => {}
                },
                _ => {}
            },
            Event::Tab => match &mut self.state.viewport {
                Viewport::Window => match &mut self.state.window {
                    Window::Library => self.state.window_state.library.lock().unwrap().tab().await?,
                    _ => {}
                },
                _ => {}
            },
            Event::Backtab => match &mut self.state.viewport {
                Viewport::Window => match &mut self.state.window {
                    Window::Library => self.state.window_state.library.lock().unwrap().backtab().await?,
                    _ => {}
                },
                _ => {}
            },
            Event::Play(play) => {
                let api = self.spotify.api.clone();
                tokio::task::spawn(async move {
                    if api.token().is_expired() {
                        api.refresh().await.unwrap();
                    }
                    api.play(play, None).await.unwrap();
                });
            }
            Event::UpdateQueue => {
                let api = self.spotify.api.clone();
                let queue = self.state.window_state.queue.clone();
                tokio::task::spawn(async move {
                    let q = api.queue().await;
                    if let Err(e) = q {
                        panic!("Failed to get queue: {:?}", e);
                    }
                    match q.ok() {
                        Some(q) => {
                            let st = api.check_saved_tracks(q
                                .queue
                                .iter()
                                .filter_map(|i| match i {
                                    Item::Track(t) => Some(t.uri.clone()),
                                    Item::Episode(_) => None,
                                })).await.unwrap();

                            let se = api.check_saved_episodes(q
                                .queue
                                .iter()
                                .filter_map(|i| match i {
                                    Item::Episode(e) => Some(e.uri.clone()),
                                    Item::Track(_) => None,
                                })).await.unwrap();
                            queue.lock().unwrap().queue =
                                Some(Queue::from((q, st, se))).into();
                        }
                        None => {
                            queue.lock().unwrap().queue = None.into();
                        }
                    }
                });
            }
            Event::Select => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices) => {
                    let device = self.state.modal_state.devices.lock().unwrap().select();
                    let api = self.spotify.api.clone();
                    tokio::task::spawn(async move {
                        if api.token().is_expired() {
                            api.refresh().await.unwrap();
                        }
                        api.transfer_playback(device.id, true).await.unwrap();
                    });
                    self.state.viewport = Viewport::Window;
                }
                #[allow(clippy::single_match)]
                Viewport::Window => match self.state.window {
                    Window::Queue => {
                        if let Some(actions) = self.state.window_state.queue.lock().unwrap().select() {
                            *self.state.modal_state.actions.lock().unwrap() = ActionState::new(actions);
                            self.state.viewport = Viewport::Modal(Modal::Action);
                        }
                    }
                    Window::Library => {
                        if let Some(actions) = self.state.window_state.library.lock().unwrap().select() {
                            *self.state.modal_state.actions.lock().unwrap() = ActionState::new(actions);
                            self.state.viewport = Viewport::Modal(Modal::Action);
                        }
                    }
                    Window::Landing => {
                        if let Some(actions) = self.state.window_state.landing.lock().unwrap().select() {
                            *self.state.modal_state.actions.lock().unwrap() = ActionState::new(actions);
                            self.state.viewport = Viewport::Modal(Modal::Action);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::OpenAction => {
                let actions = self.state.playback.lock().unwrap().into_ui_actions(true);
                if !actions.is_empty() {
                    *self.state.modal_state.actions.lock().unwrap() = ActionState::new(actions);
                    self.state.viewport = Viewport::Modal(Modal::Action);
                }
            },
            Event::OpenAddToPlaylist(uri) => {
                *self.state.modal_state.add_to_playlist.lock().unwrap() = Some(uri.clone());
                self.state.viewport = Viewport::Modal(Modal::AddToPlaylist);
            }
            Event::OpenSelectDevice => {
                if let Ok(devices) = self.spotify.api.devices().await {
                    *self.state.modal_state.devices.lock().unwrap() = DevicesState {
                        state: TableState::default(),
                        devices,
                    };
                    self.state.viewport = Viewport::Modal(Modal::Devices);
                } else {
                    return Err(eyre!("Failed to get devices"));
                }
            }
            Event::OpenGoTo => {
                self.state.viewport = Viewport::Modal(Modal::GoTo);
            }
            Event::GoTo(goto) => {
                match goto {
                    GoTo::Queue => {
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Queue;
                    }
                    GoTo::Library => {
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Library;
                    }
                    GoTo::Playlist(playlist) => {
                        *self.state.window_state.landing.lock().unwrap() = Landing::playlist(&self.spotify.api, playlist.clone()).await?;
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Landing;
                    },
                    GoTo::Album(album) => {
                        *self.state.window_state.landing.lock().unwrap() = Landing::album(&self.spotify.api, album.clone()).await?;
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Landing;
                    }
                    //GoTo::Artist(artist) => {
                    //    *self.state.window_state.landing.lock().unwrap() = Landing::artist(&self.spotify.api, artist.clone()).await?;
                    //    self.state.viewport = Viewport::Window;
                    //    self.state.window = Window::Landing;
                    //}
                    GoTo::Show(show) => {
                        *self.state.window_state.landing.lock().unwrap() = Landing::show(&self.spotify.api, show.clone()).await?;
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Landing;
                    }
                    GoTo::Audiobook(audiobook) => {
                        *self.state.window_state.landing.lock().unwrap() = Landing::audiobook(&self.spotify.api, audiobook.clone()).await?;
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Landing;
                    }
                    // TODO: Map in changing ui based on goto when the other states are implemented
                    _ => todo!(),
                }
            }
            Event::Key(key) => {
                match &mut self.state.viewport {
                    Viewport::Window => if let Some(action) = keymaps.get(&key) {
                        tx.send(action.clone()).unwrap();
                    }
                    Viewport::Modal(modal) => {
                        match key {
                            key!('q') => tx.send(Event::Close).unwrap(),
                            key!(Esc) => tx.send(Event::Close).unwrap(),
                            key!('c' + CONTROL) => tx.send(Event::Quit).unwrap(),
                            key!('C' + SHIFT + CONTROL) => tx.send(Event::Quit).unwrap(),
                            _ => match modal {
                                Modal::GoTo => {
                                    let go_to = &mut self.state.modal_state.go_to.lock().unwrap();
                                    if go_to.contains(&key) {
                                        tx.send(Event::GoTo(go_to.get(&key).unwrap().clone())).unwrap();
                                    }
                                }
                                Modal::Action => {
                                    if self.state.modal_state.actions.lock().unwrap().resolve(key, &self.spotify.api, tx.clone()) {
                                        self.state.viewport = Viewport::Window;
                                    }
                                },
                                Modal::Devices => if let Some(action) = keymaps.get(&key) {
                                    tx.send(action.clone()).unwrap();
                                }
                                _ => {},
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::ToggleRepeat => {
                if self.state.playback.lock().unwrap().is_none() {
                    return Ok(());
                }

                let (repeat, old) = {
                    let playback = self.state.playback.lock().unwrap();
                    let pb = playback.playback.as_ref().unwrap();
                    (
                        match pb.repeat {
                            Repeat::Off if !pb.disallow(PlaybackAction::TogglingRepeatTrack) => Repeat::Track,
                            Repeat::Track if !pb.disallow(PlaybackAction::TogglingRepeatContext) => Repeat::Context,
                            _ => Repeat::Off,
                        },
                        pb.repeat,
                    )
                };

                if repeat == old {
                    return Ok(());
                }

                let pb = self.state.playback.clone();
                let api = self.spotify.api.clone();
                tokio::task::spawn(async move {
                    if api.token().is_expired() {
                        api.refresh().await.unwrap();
                    }
                    api.repeat(repeat, None).await.unwrap();
                    if pb.lock().unwrap().is_some() {
                        pb.lock().unwrap().playback.as_mut().unwrap().repeat = repeat;
                    }
                });
            }
            Event::ToggleShuffle => {
                if self.state.playback.lock().unwrap().is_none() || self.state.playback.lock().unwrap().playback.as_ref().unwrap().disallow(PlaybackAction::TogglingShuffle) {
                    return Ok(());
                }

                let shuffle = !self.state.playback.lock().unwrap().playback.as_ref().unwrap().shuffle;
                let pb = self.state.playback.clone();
                let api = self.spotify.api.clone();
                tokio::task::spawn(async move {
                    if api.token().is_expired() {
                        api.refresh().await.unwrap();
                    }
                    api.shuffle(shuffle, None).await.unwrap();
                    if pb.lock().unwrap().is_some() {
                        pb.lock().unwrap().playback.as_mut().unwrap().shuffle = shuffle;
                    }
                });
            }
            Event::VolumeUp => {
                if self.state.playback.lock().unwrap().is_none() || self.state.playback.lock().unwrap().playback.as_ref().unwrap().device.is_none() {
                    return Ok(());
                }

                let (vol, unavailable) = {
                    let playback = self.state.playback.lock().unwrap();
                    let device = playback.playback.as_ref().unwrap().device.as_ref().unwrap();
                    ((device.volume_percent + 10).min(100), device.is_restricted || !device.supports_volume)
                };

                if unavailable {
                    return Ok(());
                }

                let api = self.spotify.api.clone();
                tokio::task::spawn(async move {
                    if api.token().is_expired() {
                        api.refresh().await.unwrap();
                    }
                    api.volume(vol, None).await.unwrap();
                });
            }
            Event::VolumeDown => {
                if self.state.playback.lock().unwrap().is_none() || self.state.playback.lock().unwrap().playback.as_ref().unwrap().device.is_none() {
                    return Ok(());
                }

                let (vol, unavailable) = {
                    let playback = self.state.playback.lock().unwrap();
                    let device = playback.playback.as_ref().unwrap().device.as_ref().unwrap();
                    (device.volume_percent.saturating_sub(10), device.is_restricted || !device.supports_volume)
                };

                if unavailable {
                    return Ok(());
                }

                let api = self.spotify.api.clone();
                tokio::task::spawn(async move {
                    if api.token().is_expired() {
                        api.refresh().await.unwrap();
                    }
                    api.volume(vol, None).await.unwrap();
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_events(&self, tx: mpsc::UnboundedSender<Event>) {
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
                            tx.send(Event::Key(key)).unwrap();
                          },
                          crossterm::event::Event::FocusGained => {
                            tx.send(Event::Focus).unwrap();
                          },
                          crossterm::event::Event::FocusLost => {
                            tx.send(Event::Unfocus).unwrap();
                          },
                          crossterm::event::Event::Mouse(mouse) => {
                            tx.send(Event::Mouse(mouse)).unwrap();
                          }
                          _ => {},
                        }
                      }
                      Some(Err(_)) => {
                        tx.send(Event::None).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = delay => {
                      tx.send(Event::Tick).unwrap();
                  },
                }
            }
        });
    }

    // Main Application Loop
    pub async fn run(&mut self, keymaps: HashMap<KeyEvent, Event>) -> Result<()> {
        install_hooks()?;

        tui::init()?;
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        self.handle_events(action_tx.clone());

        while !self.quit {
            // application update
            if let Some(action) = action_rx.recv().await {
                self.update(action, &keymaps, action_tx.clone()).await?;
            }
        }

        tui::restore()?;
        Ok(())
    }
}
