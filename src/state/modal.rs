use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use tokio::sync::mpsc;
use tupy::api::{flow::{AuthFlow, Pkce}, response::{Device, PagedPlaylists}, Resource, Uri, UserApi};

use crate::{app::Event, errors::LogError, key, state::actions::{Action, GoTo}, Locked, Shared};

use super::{window::Pages, IterCollection, Loading};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DevicesState {
    pub state: TableState,
    pub devices: Vec<Device>,
}

impl DevicesState {
    pub fn next(&mut self) {
        self.state.next_in_list(self.devices.len());
    }
    
    pub fn prev(&mut self) {
        self.state.prev_in_list(self.devices.len());
    }

    pub fn select(&self) -> Device {
        self.devices[self.state.selected().unwrap_or(0)].clone()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GoToState {
    lookup: HashMap<KeyEvent, usize>,
    pub mappings: Vec<(KeyEvent, GoTo)>,
}

impl GoToState {
    pub fn new(mappings: Vec<(KeyEvent, GoTo)>) -> Self {
        Self {
            lookup: mappings.iter().enumerate().map(|(i, (k, _))| (*k, i)).collect(),
            mappings,
        }
    }

    pub fn contains(&self, key: &KeyEvent) -> bool {
        self.lookup.contains_key(key)
    }

    pub fn get(&self, key: &KeyEvent) -> Option<&GoTo> {
        self.lookup.get(key).map(|i| &self.mappings[*i].1)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArtistsState {
    pub state: TableState,
    pub artists: Vec<(Uri, String)>,
}

impl ArtistsState {
    pub fn new(artists: Vec<(Uri, String)>) -> Self {
        Self {
            artists,
            state: TableState::default(),
        }
    }

    pub fn down(&mut self) {
        self.state.next_in_list(self.artists.len());
    }

    pub fn up(&mut self) {
        self.state.prev_in_list(self.artists.len());
    }

    pub fn select(&self) -> Uri {
        self.artists[self.state.selected().unwrap_or(0)].clone().0
    }
}

#[derive(Debug, Clone)]
pub struct AddToPlaylistState {
    pub item: Uri,
    pub state: TableState,
    pub playlists: Pages<PagedPlaylists, PagedPlaylists>,
}

impl AddToPlaylistState {
    pub fn new(item: Uri, playlists: Pages<PagedPlaylists, PagedPlaylists>) -> Self {
        Self {
            item,
            playlists,
            state: TableState::default(),
        }
    }

    pub fn down(&mut self) {
        if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref().map(|p| p.as_ref()){
            self.state.next_in_list(items.items.len());
        };
    }

    pub fn up(&mut self) {
        if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref().map(|p| p.as_ref()){
            self.state.prev_in_list(items.items.len());
        };
    }

    pub async fn right(&mut self) -> color_eyre::Result<()> {
        if self.playlists.has_next().await {
            self.playlists.next().await?;
        }
        Ok(())
    }

    pub async fn left(&mut self) -> color_eyre::Result<()> {
        if self.playlists.has_prev().await {
            self.playlists.prev().await?;
        }
        Ok(())
    }

    pub fn select(&self) -> Option<Uri> {
        if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref().map(|p| p.as_ref()){
            return items.items.get(self.state.selected().unwrap_or(0)).map(|p| p.uri.clone())
        }
        None
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ActionState {
    lookup: HashMap<KeyEvent, usize>,
    pub mappings: Vec<(KeyEvent, Action, &'static str)>,
}

impl ActionState {
    pub fn new(mappings: Vec<(KeyEvent, Action, &'static str)>) -> Self {
        Self {
            lookup: mappings.iter().enumerate().map(|(i, (k, _, _))| (*k, i)).collect(),
            mappings,
        }
    }

    pub fn contains(&self, key: KeyEvent) -> bool {
        self.lookup.contains_key(&key)
    }

    pub fn get(&self, key: KeyEvent) -> Option<&Action> {
        self.lookup.get(&key).map(|i| &self.mappings[*i].1)
    }

    pub fn resolve(&self, key: KeyEvent, api: &Pkce, tx: mpsc::UnboundedSender<Event>) -> bool {
        if let Some(action) = self.get(key) {
            match action {
                Action::Play(play) => {
                    let api = api.clone();
                    let uri = play.clone();
                    tokio::spawn(async move {
                        if api.token().is_expired() {
                            api.refresh().await.log_error();
                        }
                        api.add_to_queue(uri, None).await.log_error();
                        api.next(None).await.log_error();
                    });
                }
                Action::PlayContext(play) => tx.send(Event::Play(play.clone())).log_error(),
                Action::Save(uri, callback) => {
                    let api = api.clone();
                    let uri = uri.clone();
                    let callback = callback.clone();
                    match uri.resource() {
                        Resource::Track => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.save_tracks([uri]).await.log_error();
                            callback(true).log_error()
                        });},
                        Resource::Episode => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.save_episodes([uri]).await.log_error();
                            callback(true).log_error()
                        });},
                        Resource::Artist => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.follow_artists([uri]).await.log_error();
                            callback(true).log_error()
                        });},
                        Resource::Album => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.save_albums([uri]).await.log_error();
                            callback(true).log_error()
                        });},
                        Resource::Playlist => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.follow_playlist(uri, true).await.log_error();
                            callback(true).log_error()
                        });},
                        Resource::Show => {
                            tokio::spawn(async move {
                                if api.token().is_expired() {
                                    api.refresh().await.log_error();
                                }
                                api.save_shows([uri]).await.log_error();
                                callback(true).log_error()
                            });
                        },
                        _ => {}
                    }
                }
                Action::Remove(uri, callback) => {
                    let api = api.clone();
                    let uri = uri.clone();
                    let callback = callback.clone();
                    match uri.resource() {
                        Resource::Track => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.remove_saved_tracks([uri]).await.log_error();
                            callback(false).log_error()
                        });},
                        Resource::Episode => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.remove_saved_episodes([uri]).await.log_error();
                            callback(false).log_error()
                        });},
                        Resource::Artist => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.unfollow_artists([uri]).await.log_error();
                            callback(false).log_error()
                        });},
                        Resource::Album => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.remove_saved_albums([uri]).await.log_error();
                            callback(false).log_error()
                        });},
                        Resource::Playlist => {tokio::spawn(async move {
                            if api.token().is_expired() {
                                api.refresh().await.log_error();
                            }
                            api.unfollow_playlist(uri).await.log_error();
                            callback(false).log_error()
                        });},
                        Resource::Show => {
                            tokio::spawn(async move {
                                if api.token().is_expired() {
                                    api.refresh().await.log_error();
                                }
                                api.remove_saved_shows([uri]).await.log_error();
                                callback(false).log_error()
                            });
                        },
                        _ => {}
                    }
                }
                Action::AddToPlaylist(uri) => {
                    tx.send(Event::OpenAddToPlaylist(uri.clone())).log_error();
                },
                Action::AddToQueue(uri) => {
                    let api = api.clone();
                    let uri = uri.clone();
                    tokio::spawn(async move {
                        if api.token().is_expired() {
                            api.refresh().await.log_error();
                        }
                        api.add_to_queue(uri, None).await.log_error();
                    });
                },
                Action::GoTo(goto) => {
                    tx.send(Event::GoTo(goto.clone())).log_error();
                },
            };
            return true;
        }
        false
    }
}


#[derive(Debug, Clone)]
pub struct ModalState {
    pub devices: Shared<Locked<DevicesState>>,
    pub go_to: Shared<Locked<GoToState>>,
    pub actions: Shared<Locked<ActionState>>,
    pub add_to_playlist: Shared<Locked<Option<AddToPlaylistState>>>,
    pub artists: Shared<Locked<ArtistsState>>,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            devices: Shared::default(),
            go_to: Shared::new(Locked::new(GoToState::new(vec![
                (key!('_' + SHIFT), GoTo::Queue),
                (key!('L' + SHIFT), GoTo::Library),
            ]))),
            actions: Shared::default(),
            artists: Shared::default(),
            add_to_playlist: Shared::default(),
        }
    }
}

