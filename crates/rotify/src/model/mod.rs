pub mod playback;
pub mod queue;
pub mod device;
pub mod user;
pub mod tracks;
pub mod paginate;
pub mod audio;
pub mod follow;

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize};
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UriType {
    Album,
    Track,
    Playlist,
    Artist,
    Episode,
    Show,
    Audiobook,
}

impl Display for UriType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Album => "album",
            Self::Track => "track",
            Self::Playlist => "playlist",
            Self::Artist => "artist",
            Self::Episode => "episode",
            Self::Show => "show",
            Self::Audiobook => "audiobook",
        })
    }
}

impl FromStr for UriType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // FIXME: Give Error a custom error variant for invalid URIs
        match s {
            "album" => Ok(Self::Album),
            "track" => Ok(Self::Track),
            "playlist" => Ok(Self::Playlist),
            "artist" => Ok(Self::Artist),
            "episode" => Ok(Self::Episode),
            "show" => Ok(Self::Show),
            "audiobook" => Ok(Self::Audiobook),
            _ => Err(Error::Unknown("Invalid URI".to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Uri {
    uri_type: UriType,
    id: String,
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "spotify:{}:{}", self.uri_type, self.id)
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Uri {
    pub fn new(uri_type: UriType, id: String) -> Self {
        Self {
            uri_type,
            id,
        }
    }

    pub fn uri_type(&self) -> UriType {
        self.uri_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl FromStr for Uri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // FIXME: Give Error a custom error variant for invalid URIs
        let parts = s.split(':').collect::<Vec<&str>>();
        if parts.len() != 3 || parts[0] != "spotify" {
            return Err(Error::Unknown("Invalid URI".to_string()));
        }

        Ok(Self {
            uri_type: UriType::from_str(parts[1])?,
            id: parts[2].to_string(),
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct Offset {
    position: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub(crate) struct Wrapped<T>(T);
impl<T> Wrapped<T> {
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<T: Debug> Debug for Wrapped<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrapped({:?})", self.0)
    }
}

impl<'de, T> Deserialize<'de> for Wrapped<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let mut map: HashMap<String, T> = HashMap::deserialize(deserializer)?;
        if map.len() != 1 {
            return Err(serde::de::Error::custom("Expected exactly one key"));
        }

        let mut entries = map.drain();
        Ok(Wrapped(entries.next().unwrap().1))
    }
}