use crossterm::event::KeyEvent;

use rspotify::model::{Id, Type};
use serde::{Deserialize, Serialize};

use crate::{get_sized_image_url, input::Key, state::model::{Album, Artist, Playlist, Show}, uri::Uri, Error, IntoSpotifyParam};

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
        mappings: Vec<(Key, Action)>
    },
    GoTo,

    Library,
    Search,

    Playlist {
        playlist: Playlist,
        image: Option<String>
    },
    Album {
        album: Album,
        image: Option<String>
    },
    Artist {
        artist: Artist,
        image: Option<String>
    },
    Show {
        show: Show,
        image: Option<String>
    },
}

impl Open {
    pub fn devices(play: Option<bool>) -> Self {
        Self::Devices { play }
    }

    pub fn actions(mappings: impl IntoIterator<Item=(Key, Action)>) -> Self {
        Self::Actions { mappings: mappings.into_iter().collect() }
    }

    pub fn playlist(playlist: &Playlist) -> Self {
        Self::Playlist{
            playlist: playlist.clone(),
            image: get_sized_image_url(&playlist.images, 200, true)
        }
    }
    pub fn album(album: &Album) -> Self {
        Self::Album{
            album: album.clone(),
            image: get_sized_image_url(&album.images, 200, true)
        }
    }
    pub fn artist(artist: &Artist) -> Self {
        Self::Artist{
            artist: artist.clone(),
            image: get_sized_image_url(&artist.images, 200, true)
        }
    }
    pub fn show(show: &Show) -> Self {
        Self::Show{
            show: show.clone(),
            image: get_sized_image_url(&show.images, 200, true)
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
        offset: Option<Offset>,
        #[serde(default, skip_serializing_if="Option::is_none")]
        position: Option<usize>,
    },
    Uris {
        uris: Vec<Uri>,
        offset: Option<usize>,
        position: Option<usize>
    }
}

impl Play {
    pub fn label(&self) -> Result<String, Error> {
        Ok(match self {
            Self::Context { uri, offset, .. } => match offset.is_some() {
                true => "item".to_string(),
                _ => format!("{}", uri.ty)
            },
            Self::Uris { uris, .. } => {
                if uris.len() == 1 {
                    format!("{}", uris.first().unwrap().ty)
                } else {
                    "Queue".to_string()
                }
            }
        })
    }

    pub fn uris(uris: impl IntoIterator<Item=Uri>, offset: Option<usize>, position: Option<usize>) -> Self {
        Self::Uris {
            uris: uris.into_iter().collect(),
            offset,
            position,
        }
    }

    pub fn single(uri: Uri, offset: Option<usize>, position: Option<usize>) -> Self {
        Self::Uris {
            uris: vec![uri],
            offset,
            position,
        }
    }

    pub fn playlist(id: impl Id, offset: Option<Offset>, position: Option<usize>) -> Self {
        Self::Context {
            uri: Uri::new(Type::Playlist, id.id()),
            offset,
            position,
        }
    }

    pub fn artist(id: impl Id, offset: Option<Offset>, position: Option<usize>) -> Self {
        Self::Context {
            uri: Uri::new(Type::Artist, id.id()),
            offset,
            position,
        }
    }

    pub fn show(id: impl Id, offset: Option<Offset>, position: Option<usize>) -> Self
    {
        Self::Context {
            uri: Uri::new(Type::Show, id.id()),
            offset,
            position,
        }
    }

    pub fn album(id: impl Id, offset: Option<Offset>, position: Option<usize>) -> Self
    {
        Self::Context {
            uri: Uri::new(Type::Album, id.id()),
            offset,
            position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Offset {
    Position(usize),
    Uri(String),
}

impl From<Offset> for rspotify::model::Offset {
    fn from(value: Offset) -> Self {
        match value {
            Offset::Position(pos) => rspotify::model::Offset::Position(chrono::Duration::milliseconds(pos as i64)),
            Offset::Uri(uri) => rspotify::model::Offset::Uri(uri)
        }
    }
}

impl IntoSpotifyParam<rspotify::model::Offset> for Offset {
    fn into_spotify_param(self) -> rspotify::model::Offset {
        self.into()
    }
}
