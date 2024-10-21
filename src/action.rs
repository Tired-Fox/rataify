use chrono::Duration;
use crossterm::event::KeyEvent;

use rspotify::model::{Id, Offset, Type};
use serde::{Deserialize, Serialize};

use crate::{uri::Uri, IntoSpotifyParam};

#[derive(Debug, Clone, PartialEq)]
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
    Open(ModalOpen),
    /// Set the currently used device
    SetDevice(String, Option<bool>),

    /// Non mapped key
    Key(KeyEvent),

    Play(Play),
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ModalOpen {
    Devices(Option<bool>),
}

impl ModalOpen {
    pub fn devices(play: Option<bool>) -> Self {
        Self::Devices(play)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Play {
    Context(Uri, Option<Offset>, Option<Duration>),
}

impl Play {
    pub fn playlist<O, P, A, B>(id: impl Id, offset: O, position: P) -> Self
    where
        O: IntoSpotifyParam<Option<Offset>, A>,
        P: IntoSpotifyParam<Option<Duration>, B>
    {
        Self::Context(
            Uri::new(Type::Playlist, id.id()),
            offset.into_spotify_param(),
            position.into_spotify_param(),
        )
    }

    pub fn artist<O, P, A, B>(id: impl Id, offset: O, position: P) -> Self 
    where
        O: IntoSpotifyParam<Option<Offset>, A>,
        P: IntoSpotifyParam<Option<Duration>, B>
    {
        Self::Context(
            Uri::new(Type::Artist, id.id()),
            offset.into_spotify_param(),
            position.into_spotify_param(),
        )
    }

    pub fn show<O, P, A, B>(id: impl Id, offset: O, position: P) -> Self
    where
        O: IntoSpotifyParam<Option<Offset>, A>,
        P: IntoSpotifyParam<Option<Duration>, B>
    {
        Self::Context(
            Uri::new(Type::Show, id.id()),
            offset.into_spotify_param(),
            position.into_spotify_param(),
        )
    }

    pub fn album<O, P, A, B>(id: impl Id, offset: O, position: P) -> Self
    where
        O: IntoSpotifyParam<Option<Offset>, A>,
        P: IntoSpotifyParam<Option<Duration>, B>
    {
        Self::Context(
            Uri::new(Type::Album, id.id()),
            offset.into_spotify_param(),
            position.into_spotify_param(),
        )
    }
}
