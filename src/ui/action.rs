use std::fmt::Display;

use tupy::api::{Uri, request::Play};
use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum GoTo {
    Library,
    Queue,

    Album(Uri),
    Artist(Uri),
    Playlist(Uri),
    Show(Uri),
}

impl GoTo {
    pub fn with_key(&self) -> KeyCode {
        match self {
            Self::Album(_) => KeyCode::Char('B'),
            Self::Artist(_) => KeyCode::Char('A'),
            Self::Playlist(_) => KeyCode::Char('P'),
            Self::Show(_) => KeyCode::Char('S'),
            Self::Queue => KeyCode::Char('B'),
            Self::Library => KeyCode::Char('L'),
        }
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Play(Uri),
    PlayContext(Play),

    /// Saves the item to the library depending on the uri
    /// If the uri is for a context it is added to the library and if it is a track or episode it
    /// is added to the users liked/saved items
    Save(Uri),
    Remove(Uri),
    /// Opens the add to playlist modal with the uri of what is being added
    AddToPlaylist(Uri),
    /// Adds item to queue
    AddToQueue(Uri),

    GoTo(GoTo),
}

impl Action {
    pub fn with_key(&self) -> KeyCode {
        match self {
            Self::Play(_) => KeyCode::Enter,
            Self::PlayContext(_) => KeyCode::Char('c'),
            Self::Save(_) => KeyCode::Char('f'),
            Self::Remove(_) => KeyCode::Char('u'),
            Self::AddToPlaylist(_) => KeyCode::Char('p'),
            Self::AddToQueue(_) => KeyCode::Char('b'),
            Self::GoTo(goto) => goto.with_key(),
        }
    }
}

impl PartialEq<char> for &Action {
    fn eq(&self, other: &char) -> bool {
        if let KeyCode::Char(c) = self.with_key() {
            return &c == other;
        }
        false
    }
}

impl PartialEq<char> for Action {
    fn eq(&self, other: &char) -> bool {
        if let KeyCode::Char(c) = self.with_key() {
            return &c == other;
        }
        false
    }
}

impl PartialEq<KeyCode> for &Action {
    fn eq(&self, other: &KeyCode) -> bool {
        &self.with_key() == other
    }
}

impl PartialEq<KeyCode> for Action {
    fn eq(&self, other: &KeyCode) -> bool {
        &self.with_key() == other
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Play(_) => write!(f, "Play"),
            Self::PlayContext(play) => write!(f, "Play {}", match play {
                Play::Artist(_) => "Artist",
                Play::Album{..} => "Album",
                Play::Show{..} => "Show",
                Play::Playlist{..} => "Playlist",
                Play::Queue{..}  => "Queue",
                _ => "Context",
            }),
            Self::Remove(_) => write!(f, "Remove Favorite"),
            Self::Save(_) => write!(f, "Favorite"),
            Self::AddToPlaylist(_) => write!(f, "Add to Playlist"),
            Self::AddToQueue(_) => write!(f, "Add to Queue"),
            Self::GoTo(go_to) => write!(f, "Go to {}", go_to),
        }
    }
}
