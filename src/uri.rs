use rspotify::model::{parse_uri, AlbumId, ArtistId, Id, PlayContextId, PlaylistId, ShowId, Type};
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
        D: serde::Deserializer<'de>,
    {
        let uri = String::deserialize(deserializer)?;
        let (ty, id) = parse_uri(uri.as_str()).map_err(serde::de::Error::custom)?;

        Ok(Self {
            ty,
            id: id.to_string(),
        })
    }
}

impl Uri {
    pub fn new(ty: Type, id: impl std::fmt::Display) -> Self {
        Self {
            ty,
            id: id.to_string(),
        }
    }

    pub fn play_context_id(&self) -> Result<PlayContextId<'_>, Error> {
        Ok(match self.ty {
            Type::Album => PlayContextId::from(AlbumId::from_id(self.id.as_str())?),
            Type::Playlist => PlayContextId::from(PlaylistId::from_id(self.id.as_str())?),
            Type::Show => PlayContextId::from(ShowId::from_id(self.id.as_str())?),
            Type::Artist => PlayContextId::from(ArtistId::from_id(self.id.as_str())?),
            other => {
                return Err(Error::custom(format!(
                    "cannot convert {other:?} into a PlayContextId"
                )))
            }
        })
    }
}
