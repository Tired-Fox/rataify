use std::collections::HashMap;

use crossterm::event::KeyEvent;

use rspotify::model::{AlbumId, ArtistId, Id, PlaylistId, ShowId, Type};
use serde::{Deserialize, Serialize};

use crate::{input::Key, state::model::{Album, Artist, Playlist, Show}, uri::Uri, Error};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Quit application
    Quit,
    /// Close current target, if the current target is a regular layout/window then quit
    Close,

    Up,
    Down,
    Left,
    Right,

    /// Next tab
    Tab,
    /// Previous tab
    BackTab,

    /// Select current selection
    Select,
    /// Next page in pagination
    NextPage,
    /// Previous page in pagination
    PreviousPage,

    /// Go to next item in the queue
    Next,
    /// Go to previous item in the queue
    Previous,
    /// Toggle playing state item in the queue
    Toggle,
    /// Toggle shuffle state
    Shuffle,
    /// Toggle to next repeat state
    Repeat,

    /// Open a modal
    Open(Open),
    /// Set the currently used device
    SetDevice {
        id: String,
        #[serde(default, skip_serializing_if="Option::is_none")]
        play: Option<bool>
    },

    /// Non mapped key
    Key(KeyEvent),

    Play(Play),
}

impl Action {
    pub fn label(&self) -> Result<String, Error> {
        Ok(match self {
            Action::Close => "close".to_string(),
            Action::Select => "Select".to_string(),
            Action::NextPage => "Next page".to_string(),
            Action::PreviousPage => "Previous page".to_string(),
            Action::Next => "Next".to_string(),
            Action::Previous => "Previous".to_string(),
            Action::Toggle => "Toggle".to_string(),
            Action::Shuffle => "Shuffle".to_string(),
            Action::Repeat => "Repeat".to_string(),
            Action::Open(open) => format!("Open {}", open.label()?),
            Action::Play(play) => format!("Play {}", play.label()?),
            _ => return Err(Error::custom("action is not supported in the action menu"))
        })
    }
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Open {
    Devices {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        play: Option<bool>
    },
    Actions {
        mappings: HashMap<Key, Action>
    },

    Library,
    Search,

    Playlist {
        id: PlaylistId<'static>,
        name: String,
        image: Option<String>
    },
    Album {
        id: AlbumId<'static>,
        name: String,
        image: Option<String>
    },
    Artist {
        id: ArtistId<'static>,
        name: String,
        image: Option<String>
    },
    Show {
        id: ShowId<'static>,
        name: String,
        image: Option<String>
    },
}

impl Open {
    pub fn devices(play: Option<bool>) -> Self {
        Self::Devices { play }
    }

    pub fn actions(mappings: impl IntoIterator<Item=(Key, Action)>) -> Self {
        Self::Actions { mappings: HashMap::from_iter(mappings.into_iter()) }
    }

    pub fn playlist(playlist: &Playlist) -> Self {
        Self::Playlist{
            id: playlist.id.clone(),
            name: playlist.name.clone(),
            image: playlist.images.first().map(|i| i.url.clone())
        }
    }
    pub fn album(album: &Album) -> Self {
        Self::Album{
            id: album.id.clone(),
            name: album.name.clone(),
            image: album.images.first().map(|i| i.url.clone())
        }
    }
    pub fn artist(artist: &Artist) -> Self {
        Self::Artist{
            id: artist.id.clone(),
            name: artist.name.clone(),
            image: artist.images.first().map(|i| i.url.clone())
        }
    }
    pub fn show(show: &Show) -> Self {
        Self::Show{
            id: show.id.clone(),
            name: show.name.clone(),
            image: show.images.first().map(|i| i.url.clone())
        }
    }

    pub fn label(&self) -> Result<String, Error> {
        Ok(match self {
            Open::Library => "library".to_string(),
            Open::Search => "search".to_string(),
            Open::Playlist{ .. } => "playlist".to_string(),
            Open::Album{ .. } => "album".to_string(),
            Open::Artist{ .. } => "artist".to_string(),
            Open::Show{ .. } => "show".to_string(),
            other => return Err(Error::custom(format!("cannot open {other:?} from a context menu")))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Play {
    Context {
        uri: Uri,
        #[serde(default, skip_serializing_if="Option::is_none")]
        offset: Option<String>,
        #[serde(default, skip_serializing_if="Option::is_none")]
        position: Option<usize>,
    }
}

impl Play {
    pub fn label(&self) -> Result<String, Error> {
        Ok(match self {
            Self::Context { uri, .. } => format!("{}", uri.ty)
        })
    }

    pub fn playlist(id: impl Id, offset: Option<String>, position: Option<usize>) -> Self {
        Self::Context {
            uri: Uri::new(Type::Playlist, id.id()),
            offset,
            position,
        }
    }

    pub fn artist(id: impl Id, offset: Option<String>, position: Option<usize>) -> Self {
        Self::Context {
            uri: Uri::new(Type::Artist, id.id()),
            offset,
            position,
        }
    }

    pub fn show(id: impl Id, offset: Option<String>, position: Option<usize>) -> Self
    {
        Self::Context {
            uri: Uri::new(Type::Show, id.id()),
            offset,
            position,
        }
    }

    pub fn album(id: impl Id, offset: Option<String>, position: Option<usize>) -> Self
    {
        Self::Context {
            uri: Uri::new(Type::Album, id.id()),
            offset,
            position,
        }
    }
}
