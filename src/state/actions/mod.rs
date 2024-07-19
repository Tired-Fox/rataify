use std::{fmt::{Debug, Display}, rc::Rc};

use color_eyre::{eyre::eyre, Report, Result};
use crossterm::event::KeyEvent;
use tupy::api::{
    request::Play,
    response::{
        Context, Episode, Item, PlaybackItem, SimplifiedAlbum, SimplifiedChapter,
        SimplifiedEpisode, SimplifiedTrack, Track,
    },
    Resource, Uri, UserResource,
};

use crate::{key, Shared};

use super::{playback::PlaybackState, wrappers::GetUri};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum GoTo {
    Library,
    Queue,
    LikedSongs,
    MyEpisodes,

    Album(Uri),
    Artist(Uri),
    Artists(Vec<(Uri, String)>),
    Playlist(Uri),
    Show(Uri),
    Audiobook(Uri),
}

impl TryFrom<Uri> for GoTo {
    type Error = Report;

    fn try_from(value: Uri) -> Result<Self> {
        Ok(match value.resource() {
            Resource::Playlist => Self::Playlist(value),
            Resource::Album => Self::Album(value),
            Resource::Show => Self::Show(value),
            Resource::Artist => Self::Artist(value),
            Resource::User(user_resource) => match user_resource {
                // TODO: May need more information in this action to get the correct page
                UserResource::Collection => Self::LikedSongs,
                UserResource::CollectionYourEpisodes => Self::MyEpisodes,
                _ => {
                    return Err(eyre!(
                        "Invalid uri cannot be convert to a GoTo action: {value}"
                    ))
                }
            },
            _ => {
                return Err(eyre!(
                    "Invalid uri cannot be convert to a GoTo action: {value}"
                ))
            }
        })
    }
}

impl Display for GoTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Album(_) => write!(f, "Album"),
            Self::Artist(_) => write!(f, "Artist"),
            Self::Playlist(_) => write!(f, "Playlist"),
            Self::Show(_) => write!(f, "Show"),
            Self::Queue => write!(f, "Queue"),
            Self::Library => write!(f, "Library"),
            Self::Audiobook(_) => write!(f, "Audiobook"),
            Self::LikedSongs => write!(f, "Liked Songs"),
            Self::MyEpisodes => write!(f, "My Episodes"),
            Self::Artists(_) => write!(f, "Artists"),
        }
    }
}

pub mod action_label {
    pub static ADD_TO_PLAYLIST: &str = "Add to Playlist";
    pub static ADD_TO_QUEUE: &str = "Add to Queue";

    pub static PLAY: &str = "Play";
    pub static REMOVE: &str = "Remove";
    pub static SAVE: &str = "Save";

    pub static GO_TO_PLAYLIST: &str = "Go to Playlist";
    pub static PLAY_PLAYLIST: &str = "Play Playlist";

    pub static GO_TO_ALBUM: &str = "Go to Album";
    pub static PLAY_ALBUM: &str = "Play Album";

    pub static GO_TO_SHOW: &str = "Go to Show";
    pub static PLAY_SHOW: &str = "Play Show";

    pub static GO_TO_ARTIST: &str = "Go to Artist";
    pub static SELECT_ARTIST: &str = "Select an Artist";

    pub static GO_TO_AUDIOBOOK: &str = "Go to Audiobook";
    pub static GO_TO_CONTEXT: &str = "Go to Context";
}

#[derive(Clone)]
pub enum Action {
    Play(Uri),
    PlayContext(Play),

    /// Saves the item to the library depending on the uri
    /// If the uri is for a context it is added to the library and if it is a track or episode it
    /// is added to the users liked/saved items
    Save(Uri, Shared<dyn Fn(bool) -> Result<()> + Sync + Send>),
    Remove(Uri, Shared<dyn Fn(bool) -> Result<()> + Sync + Send>),
    /// Opens the add to playlist modal with the uri of what is being added
    AddToPlaylist(Uri),
    /// Adds item to queue
    AddToQueue(Uri),

    GoTo(GoTo),
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Play(u1) => if let Action::Play(u2) = other { u1.eq(u2) } else { false },
            Self::PlayContext(p1) => if let Action::PlayContext(p2) = other { p1.eq(p2) } else { false },
            Self::AddToPlaylist(u1) => if let Action::AddToPlaylist(u2) = other { u1.eq(u2) } else { false },
            Self::AddToQueue(u1) => if let Action::AddToQueue(u2) = other { u1.eq(u2) } else { false },
            Self::GoTo(g1) => if let Action::GoTo(g2) = other { g1.eq(g2) } else { false },
            Self::Save(u1, _) =>   if let Action::Save(u2, _) = other { u1.eq(u2) } else { false },
            Self::Remove(u1, _) => if let Action::Remove(u2, _) = other { u1.eq(u2) } else { false },
        }
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Play(u) => write!(f, "Play({})", u),
            Self::PlayContext(p) => write!(f, "PlayContext({:?})", p),
            Self::AddToPlaylist(u) => write!(f, "AddToPlaylist({})", u),
            Self::AddToQueue(u) => write!(f, "AddToQueue({})", u),
            Self::GoTo(g) => write!(f, "GoTo({:?})", g),
            Self::Save(u, _) => write!(f, "Save({})", u),
            Self::Remove(u, _) => write!(f, "Remove({})", u),
        }
    }
}

impl Action {
    pub fn save<F>(uri: Uri, callback: F) -> Self
    where
        F: Fn(bool) -> Result<()> + Sync + Send + 'static
    {
        Self::Save(uri, Shared::new(callback))
    } 

    pub fn remove<F>(uri: Uri, callback: F) -> Self
    where
        F: Fn(bool) -> Result<()> + Sync + Send + 'static
    {
        Self::Remove(uri, Shared::new(callback))
    } 
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Play(_) => write!(f, "Play"),
            Self::PlayContext(play) => write!(
                f,
                "Play {}",
                match play {
                    Play::Artist(_) => "Artist",
                    Play::Album { .. } => "Album",
                    Play::Show { .. } => "Show",
                    Play::Playlist { .. } => "Playlist",
                    Play::Queue { .. } => "Queue",
                    _ => "Context",
                }
            ),
            Self::Remove(_, _) => write!(f, "Remove Favorite"),
            Self::Save(_, _) => write!(f, "Favorite"),
            Self::AddToPlaylist(_) => write!(f, "Add to Playlist"),
            Self::AddToQueue(_) => write!(f, "Add to Queue"),
            Self::GoTo(go_to) => write!(f, "Go to {}", go_to),
        }
    }
}

pub trait IntoActions {
    fn into_ui_actions(&self, context: bool) -> Vec<(KeyEvent, Action, &'static str)>;
}

impl IntoActions for SimplifiedAlbum {
    fn into_ui_actions(&self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let mut actions = vec![
            (
                key!('C'),
                Action::GoTo(GoTo::Album(self.uri.clone())),
                action_label::GO_TO_ALBUM,
            ),
        ];

        if self.artists.len() > 1 {
            actions.push((
                key!('A'),
                Action::GoTo(GoTo::Artists(
                    self.artists
                        .iter()
                        .map(|a| (a.uri.clone(), a.name.clone()))
                        .collect::<Vec<_>>(),
                )),
                action_label::SELECT_ARTIST,
            ))
        }

        actions
    }
}

impl IntoActions for Track {
    fn into_ui_actions(&self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let mut actions = vec![
            (
                key!('p'),
                Action::AddToPlaylist(self.uri.clone()),
                action_label::ADD_TO_PLAYLIST,
            ),
            (
                key!('b'),
                Action::AddToQueue(self.uri.clone()),
                action_label::ADD_TO_QUEUE,
            ),
            if self.artists.len() == 1 {
                (
                    key!('A'),
                    Action::GoTo(GoTo::Artist(self.artists.first().unwrap().uri.clone())),
                    action_label::GO_TO_ARTIST,
                )
            } else {
                (
                    key!('A'),
                    Action::GoTo(GoTo::Artists(
                        self.artists
                            .iter()
                            .map(|a| (a.uri.clone(), a.name.clone()))
                            .collect::<Vec<_>>(),
                    )),
                    action_label::SELECT_ARTIST,
                )
            },
        ];

        if context {
            if self.album.total_tracks > 1 {
                actions.push((
                    key!('c'),
                    Action::PlayContext(Play::album(self.album.uri.clone(), None, 0)),
                    action_label::PLAY_ALBUM,
                ));
            }
            actions.push((
                key!('C'),
                Action::GoTo(GoTo::Album(self.album.uri.clone())),
                action_label::GO_TO_ALBUM,
            ))
        }

        actions
    }
}

impl IntoActions for SimplifiedTrack {
    fn into_ui_actions(&self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let actions = vec![
            (
                key!('p'),
                Action::AddToPlaylist(self.uri.clone()),
                action_label::ADD_TO_PLAYLIST,
            ),
            (
                key!('b'),
                Action::AddToQueue(self.uri.clone()),
                action_label::ADD_TO_QUEUE,
            ),
            if self.artists.len() == 1 {
                (
                    key!('A'),
                    Action::GoTo(GoTo::Artist(self.artists.first().unwrap().uri.clone())),
                    action_label::GO_TO_ARTIST,
                )
            } else {
                (
                    key!('A'),
                    Action::GoTo(GoTo::Artists(
                        self.artists
                            .iter()
                            .map(|a| (a.uri.clone(), a.name.clone()))
                            .collect::<Vec<_>>(),
                    )),
                    action_label::SELECT_ARTIST,
                )
            },
        ];

        actions
    }
}

impl IntoActions for SimplifiedEpisode {
    fn into_ui_actions(&self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        vec![
            (
                key!('p'),
                Action::AddToPlaylist(self.uri.clone()),
                action_label::ADD_TO_PLAYLIST,
            ),
            (
                key!('b'),
                Action::AddToQueue(self.uri.clone()),
                action_label::ADD_TO_QUEUE,
            ),
        ]
    }
}

impl IntoActions for Episode {
    fn into_ui_actions(&self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let mut actions = vec![
            (
                key!('p'),
                Action::AddToPlaylist(self.uri.clone()),
                action_label::ADD_TO_PLAYLIST,
            ),
            (
                key!('b'),
                Action::AddToQueue(self.uri.clone()),
                action_label::ADD_TO_QUEUE,
            ),
        ];

        if context {
            if let Some(show) = self.show.as_ref() {
                if show.total_episodes > 1 {
                    actions.push((
                        key!('c'),
                        Action::PlayContext(Play::show(show.uri.clone(), None, 0)),
                        action_label::PLAY_SHOW,
                    ));
                }
                actions.push((
                    key!('C'),
                    Action::GoTo(GoTo::Show(show.uri.clone())),
                    action_label::GO_TO_SHOW,
                ));
            }
        }

        actions
    }
}

impl IntoActions for SimplifiedChapter {
    fn into_ui_actions(&self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        vec![
            (
                key!('p'),
                Action::AddToPlaylist(self.uri.clone()),
                action_label::ADD_TO_PLAYLIST,
            ),
            (
                key!('b'),
                Action::AddToQueue(self.uri.clone()),
                action_label::ADD_TO_QUEUE,
            ),
        ]
    }
}

impl IntoActions for Item {
    fn into_ui_actions(&self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        match self {
            Item::Track(t) => t.into_ui_actions(context),
            Item::Episode(e) => e.into_ui_actions(context),
        }
    }
}

impl GetUri for Item {
    fn get_uri(&self) -> Uri {
        match self {
            Item::Track(t) => t.uri.clone(),
            Item::Episode(e) => e.uri.clone(),
        }
    }
}

impl IntoActions for PlaybackState {
    fn into_ui_actions(&self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        if let Some(pb) = self.playback.as_ref() {
            match &pb.item {
                PlaybackItem::Track(t) => {
                    let mut actions = vec![
                        // TODO: Wrap the playback fetching on if it is saved. If it has the
                        // functionality then add the action to save/remove it from saved items
                        if !pb.saved {
                            (key!('f'), Action::save(t.uri.clone(), |saved| Ok(())), action_label::SAVE)
                        } else {
                            (
                                key!('r'),
                                Action::remove(t.uri.clone(), |saved| Ok(())),
                                action_label::REMOVE,
                            )
                        },
                        (
                            key!('p'),
                            Action::AddToPlaylist(t.uri.clone()),
                            action_label::ADD_TO_PLAYLIST,
                        ),
                    ];

                    if context {
                        if t.album.total_tracks > 1 {
                            actions.push((
                                key!('c'),
                                Action::PlayContext(Play::album(t.album.uri.clone(), None, 0)),
                                action_label::PLAY_ALBUM,
                            ));
                        }
                        actions.push((
                            key!('A'),
                            Action::GoTo(GoTo::Album(t.album.uri.clone())),
                            action_label::GO_TO_ALBUM,
                        ));
                        match pb.context.as_ref() {
                            Some(Context { uri, .. }) => actions.push((
                                key!('C'),
                                Action::GoTo(GoTo::try_from(uri.clone()).unwrap()),
                                action_label::GO_TO_CONTEXT,
                            )),
                            None => {}
                        }
                    }

                    actions
                }
                PlaybackItem::Episode(e) => {
                    let mut actions = vec![
                        if !pb.saved {
                            (key!('f'), Action::save(e.uri.clone(), |saved| Ok(())), action_label::SAVE)
                        } else {
                            (
                                key!('r'),
                                Action::remove(e.uri.clone(), |saved| Ok(())),
                                action_label::REMOVE,
                            )
                        },
                        (
                            key!('p'),
                            Action::AddToPlaylist(e.uri.clone()),
                            action_label::ADD_TO_PLAYLIST,
                        ),
                    ];
                    if context {
                        if let Some(show) = e.show.as_ref() {
                            if show.total_episodes > 1 {
                                actions.push((
                                    key!('c'),
                                    Action::PlayContext(Play::show(show.uri.clone(), None, 0)),
                                    action_label::PLAY_SHOW,
                                ));
                            }
                            actions.push((
                                key!('C'),
                                Action::GoTo(GoTo::Show(show.uri.clone())),
                                action_label::GO_TO_SHOW,
                            ));
                        }
                    }
                    actions
                }
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }
}
