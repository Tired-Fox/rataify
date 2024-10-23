use std::str::FromStr;

use rspotify::model::{parse_uri, AlbumId, ArtistId, EpisodeId, Id, PlayContextId, PlayableId, PlaylistId, ShowId, TrackId, Type};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Uri {
    pub ty: Type,
    pub id: String,
}

impl<I: Id> From<I> for Uri {
    fn from(value: I) -> Self {
        Self {
            ty: value._type(),
            id: value.id().to_string(),
        }
    }
}

impl FromStr for Uri {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ty, id) = parse_uri(s)?;

        Ok(Self {
            ty,
            id: id.to_string(),
        })
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ty {
            Type::Collection => write!(f, "spotify:user:{}:collection", self.id),
            Type::Collectionyourepisodes => {
                write!(f, "spotify:user:{}:collection:your-episodes", self.id)
            }
            other => write!(f, "spotify:{}:{}", other, self.id),
        }
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let uri = self.to_string();
        serializer.serialize_str(uri.as_str())
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let uri = String::deserialize(deserializer)?;
        Self::from_str(uri.as_str()).map_err(serde::de::Error::custom)
    }
}

impl Uri {
    pub fn new(ty: Type, id: impl std::fmt::Display) -> Self {
        Self {
            ty,
            id: id.to_string(),
        }
    }

    pub fn play_context_id(&self) -> Result<PlayContextId<'static>, Error> {
        Ok(match self.ty {
            Type::Album => PlayContextId::from(AlbumId::from_id(self.id.clone())?),
            Type::Playlist => PlayContextId::from(PlaylistId::from_id(self.id.clone())?),
            Type::Show => PlayContextId::from(ShowId::from_id(self.id.clone())?),
            Type::Artist => PlayContextId::from(ArtistId::from_id(self.id.clone())?),
            other => {
                return Err(Error::custom(format!(
                    "cannot convert {other:?} into a PlayContextId"
                )))
            }
        })
    }

    pub fn playable_id(&self) -> Result<PlayableId<'static>, Error> {
        Ok(match self.ty {
            Type::Track => PlayableId::from(TrackId::from_id(self.id.clone())?),
            Type::Episode => PlayableId::from(EpisodeId::from_id(self.id.clone())?),
            other => {
                return Err(Error::custom(format!(
                    "cannot convert {other:?} into a PlayContextId"
                )))
            }
        })
    }
}
