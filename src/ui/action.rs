use std::fmt::Display;

use tupy::api::Uri;

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
    pub fn with_key(&self) -> char {
        match self {
            Self::Album(_) => 'B',
            Self::Artist(_) => 'A',
            Self::Playlist(_) => 'P',
            Self::Show(_) => 'S',
            Self::Queue => '_',
            Self::Library => 'L',
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
pub enum UiAction {
    Play(Uri),
    PlayContext(Uri),

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

impl UiAction {
    pub fn with_key(&self) -> char {
        match self {
            Self::Play(_) => 'p',
            Self::PlayContext(_) => 'c',
            Self::Save(_) => 'f',
            Self::Remove(_) => 'u',
            Self::AddToPlaylist(_) => 'a',
            Self::AddToQueue(_) => 'b',
            Self::GoTo(goto) => goto.with_key(),
        }
    }
}

impl PartialEq<char> for &UiAction {
    fn eq(&self, other: &char) -> bool {
        &self.with_key() == other
    }
}

impl PartialEq<char> for UiAction {
    fn eq(&self, other: &char) -> bool {
        &self.with_key() == other
    }
}

impl Display for UiAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Play(_) => write!(f, "Play"),
            Self::PlayContext(uri) => write!(f, "Play {:?}", uri.resource()),
            Self::Remove(_) => write!(f, "Remove Favorite"),
            Self::Save(_) => write!(f, "Favorite"),
            Self::AddToPlaylist(_) => write!(f, "Add to Playlist"),
            Self::AddToQueue(_) => write!(f, "Add to Queue"),
            Self::GoTo(go_to) => write!(f, "Go to {}", go_to),
        }
    }
}
