use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use tokio::sync::mpsc;
use tupy::api::{flow::{AuthFlow, Pkce}, response::Device, Resource, Uri, UserApi};

use crate::{app::Event, key, ui::action::{Action, GoTo}, Locked, Shared};

use super::IterCollection;

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
pub struct ActionState {
    lookup: HashMap<KeyEvent, usize>,
    pub mappings: Vec<(KeyEvent, Action)>,
}

impl ActionState {
    pub fn new(mappings: Vec<(KeyEvent, Action)>) -> Self {
        Self {
            lookup: mappings.iter().enumerate().map(|(i, (k, _))| (*k, i)).collect(),
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
                    tokio::task::spawn(async move {
                        if api.token().is_expired() {
                            api.refresh().await.unwrap();
                        }
                        api.add_to_queue(uri, None).await.unwrap();
                        api.next(None).await.unwrap();
                    });
                }
                Action::PlayContext(play) => tx.send(Event::Play(play.clone())).unwrap(),
                Action::Save(uri) => {
                    let api = api.clone();
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
                Action::Remove(uri) => {
                    let api = api.clone();
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
                Action::AddToPlaylist(uri) => {
                    tx.send(Event::OpenAddToPlaylist(uri.clone())).unwrap();
                },
                Action::AddToQueue(uri) => {
                    let api = api.clone();
                    let uri = uri.clone();
                    tokio::task::spawn(async move {
                        if api.token().is_expired() {
                            api.refresh().await.unwrap();
                        }
                        api.add_to_queue(uri, None).await.unwrap();
                    });
                },
                Action::GoTo(goto) => {
                    tx.send(Event::GoTo(goto.clone())).unwrap();
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
    pub add_to_playlist: Shared<Locked<Option<Uri>>>,
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
            add_to_playlist: Shared::default(),
        }
    }
}

