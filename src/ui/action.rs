use std::fmt::Display;

use color_eyre::{Result, eyre::eyre, Report};
use tupy::api::{request::Play, Resource, Uri};

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
            _ => return Err(eyre!("Invalid uri cannot be convert to a GoTo action: {value}"))
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

pub mod ActionLabel {
    pub static AddToPlaylist: &str = "Add to Playlist";
    pub static AddToQueue: &str = "Add to Queue";

    pub static Play: &str = "Play";
    pub static Remove: &str = "Remove";
    pub static Save: &str = "Save";

    pub static GoToPlaylist: &str = "Go to Playlist";
    pub static PlayPlaylist: &str = "Play Playlist";

    pub static GoToAlbum: &str = "Go to Album";
    pub static PlayAlbum: &str = "Play Album";

    pub static GoToShow: &str = "Go to Show";
    pub static PlayShow: &str = "Play Show";

    pub static GoToArtist: &str = "Go to Artist";
    pub static SelectArtist: &str = "Select an Artist";

    pub static GoToAudiobook: &str = "Go to Audiobook";
    pub static GoToContext: &str = "Go to Context";
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
