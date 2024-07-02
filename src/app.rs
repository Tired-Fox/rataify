use std::{collections::HashMap, io::stderr};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    symbols::border,
    widgets::{
        block::{Position, Title},
        Block, StatefulWidget, TableState, Widget,
    },
    Terminal,
};
use tokio::sync::mpsc;

use tupy::{
    api::{
        flow::{AuthFlow, Credentials, Pkce}, request::Play, response::{Item, PlaybackItem}, scopes, OAuth, Resource, Spotify, UserApi
    },
    Duration, Local,
};

use crate::{
    errors::install_hooks,
    spotify_util::listen_for_authentication_code,
    state::{Countdown, DevicesState, Loading, Modal, Queue, State, Viewport, Window},
    tui,
    ui::{action::{GoTo, UiAction}, modal::{actions::ModalActions, goto::UiGoto, add_to_playlist::AddToPlaylist}, NoPlayback},
};

static FPS: u8 = 24;
lazy_static::lazy_static! {
    static ref FPS_DURATION: Duration = Duration::milliseconds((1.0 / FPS as f32) as i64 * 1000);
}

#[derive(Debug, Clone)]
pub enum Action {
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

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Select,

    // Open menu
    OpenSelectDevice,
    OpenGoTo,
    OpenAction,
    OpenHelp,
    OpenSearch,

    // Misc input events
    Key(KeyCode),
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
            scopes::USER_READ_CURRENTLY_PLAYING,
            scopes::USER_READ_PLAYBACK_STATE,
            scopes::USER_MODIFY_PLAYBACK_STATE,
            scopes::USER_READ_PLAYBACK_POSITION,
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

        if app.spotify.api.scopes() != &app.spotify.api.token().scopes {
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

        *app.state.playback.lock().unwrap() = app
            .spotify
            .api
            .playback_state(None)
            .await?
            .map(|pb| pb.into());
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
                self.render()?;

                // Poll to check if playback state should be fetched
                if self.state.playback_poll.poll() {
                    let playback = self.state.playback.clone();
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
                            *playback.lock().unwrap() = result.map(|pb| pb.into());
                            *last_playback_poll.lock().unwrap() = Local::now();
                        }
                    });

                    tx.send(Action::UpdateQueue).unwrap();
                    self.state.playback_poll.reset();
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

                tx.send(Action::OpenSelectDevice)?;
            }
            Action::Down => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices) => {
                    self.state.modal_state.lock().unwrap().devices.next()
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Queue => self.state.window_state.lock().unwrap().queue.next(),
                    _ => {}
                },
                _ => {}
            },
            Action::Up => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices) => {
                    self.state.modal_state.lock().unwrap().devices.prev()
                }
                Viewport::Window => match &mut self.state.window {
                    Window::Queue => self.state.window_state.lock().unwrap().queue.prev(),
                    _ => {}
                },
                _ => {}
            },
            Action::Play(play) => {
                let api = self.spotify.api.clone();
                tokio::task::spawn(async move {
                    if api.token().is_expired() {
                        api.refresh().await.unwrap();
                    }
                    api.play(play, None).await.unwrap();
                });
            }
            Action::UpdateQueue => {
                let api = self.spotify.api.clone();
                let window = self.state.window_state.clone();
                tokio::task::spawn(async move {
                    match api.queue().await.ok() {
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

                            window.lock().unwrap().queue.queue =
                                Some(Queue::from((q, st, se))).into();
                        }
                        None => {
                            window.lock().unwrap().queue.queue = None.into();
                        }
                    }
                });
            }
            Action::Select => match &mut self.state.viewport {
                Viewport::Modal(Modal::Devices) => {
                    let device = self.state.modal_state.lock().unwrap().devices.select();
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
                        if let Some(item) = self.state.window_state.lock().unwrap().queue.select() {
                            self.state.modal_state.lock().unwrap().actions = match item.item {
                                Item::Track(t) => {
                                    let mut actions = vec![
                                        UiAction::Play(t.uri.clone()),
                                        if !item.saved { UiAction::Save(t.uri.clone()) } else { UiAction::Remove(t.uri.clone()) },
                                        UiAction::AddToPlaylist(t.uri.clone()),
                                        UiAction::AddToQueue(t.uri.clone()),
                                    ];
                                    if t.album.total_tracks > 1 {
                                        actions.push(UiAction::PlayContext(t.album.uri.clone()));
                                    }
                                    actions
                                }
                                Item::Episode(e) => {
                                    let mut actions = vec![
                                        UiAction::Play(e.uri.clone()),
                                        if !item.saved { UiAction::Save(e.uri.clone()) } else { UiAction::Remove(e.uri.clone()) },
                                        UiAction::AddToPlaylist(e.uri.clone()),
                                        UiAction::AddToQueue(e.uri.clone()),
                                    ];
                                    if let Some(show) = e.show.as_ref() {
                                        if show.total_episodes > 1 {
                                            actions.push(UiAction::PlayContext(show.uri.clone()));
                                        }
                                        actions.push(UiAction::GoTo(GoTo::Show(show.uri.clone())));
                                    }
                                    actions
                                }
                            };
                            self.state.viewport = Viewport::Modal(Modal::Action);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            #[allow(clippy::single_match)]
            Action::OpenAction => if let Some(playback) = self.state.playback.lock().unwrap().as_ref() {
                match &playback.item {
                    PlaybackItem::Track(t) => {
                        let mut actions = vec![
                            // TODO: Wrap the playback fetching on if it is saved. If it has the
                            // functionality then add the action to save/remove it from saved items
                            
                            //if !item.saved { UiAction::Save(t.uri.clone()) } else { UiAction::Remove(t.uri.clone()) },
                            UiAction::AddToPlaylist(t.uri.clone()),
                        ];

                        if t.album.total_tracks > 1 {
                            actions.push(UiAction::PlayContext(t.album.uri.clone()));
                        }

                        self.state.modal_state.lock().unwrap().actions = actions;
                        self.state.viewport = Viewport::Modal(Modal::Action);
                    }
                    PlaybackItem::Episode(e) => {
                        let mut actions = vec![
                            //if !item.saved { UiAction::Save(e.uri.clone()) } else { UiAction::Remove(e.uri.clone()) },
                            UiAction::AddToPlaylist(e.uri.clone()),
                            UiAction::AddToQueue(e.uri.clone()),
                        ];
                        if let Some(show) = e.show.as_ref() {
                            if show.total_episodes > 1 {
                                actions.push(UiAction::PlayContext(show.uri.clone()));
                            }
                            actions.push(UiAction::GoTo(GoTo::Show(show.uri.clone())));
                        }

                        self.state.modal_state.lock().unwrap().actions = actions;
                        self.state.viewport = Viewport::Modal(Modal::Action);
                    },
                    _ => {} 
                };
            },
            Action::OpenSelectDevice => {
                if let Ok(devices) = self.spotify.api.devices().await {
                    self.state.modal_state.lock().unwrap().devices = DevicesState {
                        state: TableState::default(),
                        devices,
                    };
                    self.state.viewport = Viewport::Modal(Modal::Devices);
                } else {
                    return Err(eyre!("Failed to get devices"));
                }
            }
            Action::OpenGoTo => {
                self.state.viewport = Viewport::Modal(Modal::GoTo);
            }
            Action::GoTo(goto) => {
                match goto {
                    GoTo::Queue => {
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Queue;
                    }
                    GoTo::Library => {
                        self.state.viewport = Viewport::Window;
                        self.state.window = Window::Library;
                    }
                    // TODO: Map in changing ui based on goto when the other states are implemented
                    _ => todo!(),
                }
            }
            Action::Key(key) => {
                match &mut self.state.viewport {
                    Viewport::Modal(Modal::GoTo) => {
                        let go_to = &mut self.state.modal_state.lock().unwrap().go_to;
                        if go_to.contains(&key) {
                            tx.send(Action::GoTo(go_to.get(&key).unwrap().clone())).unwrap();
                        }
                    }
                    Viewport::Modal(Modal::Action) => if let KeyCode::Char(key) = key {
                        let action = self.state.modal_state.lock().unwrap().actions.iter().find(|a| *a == key).cloned();
                        if let Some(action) = action {
                            match action {
                                UiAction::Play(uri) => tx.send(Action::Play(Play::queue([uri.clone()], None, 0))).unwrap(),
                                UiAction::PlayContext(uri) => match uri.resource() {
                                    Resource::Playlist => tx.send(Action::Play(Play::playlist(uri.clone(), None, 0))).unwrap(),
                                    Resource::Artist => tx.send(Action::Play(Play::artist(uri.clone()))).unwrap(),
                                    Resource::Album => tx.send(Action::Play(Play::album(uri.clone(), None, 0))).unwrap(),
                                    _ => {}
                                }
                                UiAction::Save(uri) => {
                                    let api = self.spotify.api.clone();
                                    let uri = uri.clone();
                                    match uri.resource() {
                                        Resource::Track => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.save_tracks([uri]).await.unwrap();
                                        });},
                                        Resource::Episode => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.save_episodes([uri]).await.unwrap();
                                        });},
                                        Resource::Artist => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.follow_artists([uri]).await.unwrap();
                                        });},
                                        Resource::Album => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.save_albums([uri]).await.unwrap();
                                        });},
                                        Resource::Playlist => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.follow_playlist(uri, true).await.unwrap();
                                        });},
                                        Resource::Show => {
                                            tokio::task::spawn(async move {
                                                if api.token().is_expired() {
                                                    api.refresh().await.unwrap();
                                                }
                                                api.save_shows([uri]).await.unwrap();
                                            });
                                        },
                                        _ => {}
                                    }
                                }
                                UiAction::Remove(uri) => {
                                    let api = self.spotify.api.clone();
                                    let uri = uri.clone();
                                    match uri.resource() {
                                        Resource::Track => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.remove_saved_tracks([uri]).await.unwrap();
                                        });},
                                        Resource::Episode => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.remove_saved_episodes([uri]).await.unwrap();
                                        });},
                                        Resource::Artist => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.unfollow_artists([uri]).await.unwrap();
                                        });},
                                        Resource::Album => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.remove_saved_albums([uri]).await.unwrap();
                                        });},
                                        Resource::Playlist => {tokio::task::spawn(async move {
                                            if api.token().is_expired() {
                                                api.refresh().await.unwrap();
                                            }
                                            api.unfollow_playlist(uri).await.unwrap();
                                        });},
                                        Resource::Show => {
                                            tokio::task::spawn(async move {
                                                if api.token().is_expired() {
                                                    api.refresh().await.unwrap();
                                                }
                                                api.remove_saved_shows([uri]).await.unwrap();
                                            });
                                        },
                                        _ => {}
                                    }
                                }
                                UiAction::AddToPlaylist(uri) => {
                                    self.state.modal_state.lock().unwrap().add_to_playlist = Some(uri.clone());
                                    self.state.viewport = Viewport::Modal(Modal::AddToPlaylist);
                                },
                                UiAction::AddToQueue(uri) => {
                                    let api = self.spotify.api.clone();
                                    let uri = uri.clone();
                                    tokio::task::spawn(async move {
                                        if api.token().is_expired() {
                                            api.refresh().await.unwrap();
                                        }
                                        api.add_to_queue(uri, None).await.unwrap();
                                    });
                                },
                                UiAction::GoTo(goto) => {
                                    tx.send(Action::GoTo(goto.clone())).unwrap();
                                },
                            };
                            self.state.viewport = Viewport::Window;
                        }
                    }
                    _ => {}
                }
            }
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
                                tx.send(action.clone()).unwrap();
                              } else {
                                tx.send(Action::Key(key.code)).unwrap();
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

        //let mut dimmed = if let Viewport::Modal(_) = &self.viewport {
        //    Style::default().dim()
        //} else {
        //    Style::default()
        //};
        let mut dimmed = Style::default();

        match &mut self.window {
            Window::Queue => {
                let qstate = &mut self.window_state.lock().unwrap().queue;
                StatefulWidget::render(qstate, layout[0], buf, &mut dimmed);
            }
            Window::Library => {
                Block::bordered()
                    .title(
                        Title::from("[Library]")
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    )
                    .border_set(border::ROUNDED)
                    .render(layout[0], buf);
            }
        }

        // Viewport State Rendering
        if let Viewport::Modal(modal) = &mut self.viewport {
            match modal {
                Modal::Devices => {
                    let devices = &mut self.modal_state.lock().unwrap().devices;
                    Widget::render(devices, layout[0], buf);
                }
                Modal::GoTo => {
                    let goto = &self.modal_state.lock().unwrap().go_to;
                    Widget::render(UiGoto(&goto.mappings), layout[0], buf);
                },
                Modal::Action => {
                    let actions = &self.modal_state.lock().unwrap().actions;
                    Widget::render(ModalActions(actions), layout[0], buf);
                }
                Modal::AddToPlaylist => {
                    let add_to_playlist = &self.modal_state.lock().unwrap().add_to_playlist;
                    // TODO:
                    // - Fetch playlists
                    // - Render playlists in modal
                    // - Use loading state
                    Widget::render(AddToPlaylist(add_to_playlist.as_ref()), layout[0], buf);
                }
            }
        }

        match self.playback.lock().unwrap().as_ref() {
            Some(playback) => StatefulWidget::render(
                playback,
                layout[1],
                buf,
                &mut self.last_playback_poll.lock().unwrap().clone(),
            ),
            None => {
                NoPlayback.render(layout[1], buf);
            }
        }
    }
}
