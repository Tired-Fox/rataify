use crossterm::event::KeyEvent;

use rspotify::model::{Id, Type};
use serde::{Deserialize, Serialize};

use crate::uri::Uri;

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


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Open {
    Devices {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        play: Option<bool>
    },
}

impl Open {
    pub fn devices(play: Option<bool>) -> Self {
        Self::Devices { play }
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
