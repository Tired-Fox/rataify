use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use rspotify::model::{parse_uri, AlbumId, ArtistId, Id, PlayContextId, PlaylistId, ShowId, Type};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    modifiers: KeyModifiers,
    key: KeyCode,
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut parts = Vec::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
           parts.push("ctrl".to_string())
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
           parts.push("alt".to_string())
        }
        parts.push(self.key.to_string().to_ascii_lowercase());
        
        serializer.serialize_str(parts.join("+").as_str())
    }
}

impl<'de> Deserialize<'de> for Key {
   fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
       where
           D: serde::Deserializer<'de> {
       let key = String::deserialize(deserializer)?;

       let mut modifiers = KeyModifiers::empty();
       let parts = key.split("+").collect::<Vec<_>>();
      for part in &parts[..parts.len()-1] {
         match part.to_ascii_lowercase().as_str() {
            "ctrl" => modifiers.insert(KeyModifiers::CONTROL),
            "alt" => modifiers.insert(KeyModifiers::CONTROL),
            "shift" => {},
            other => return Err(serde::de::Error::custom(format!("unknown key modifier {other}")))
         }
      }

      let last = parts.last().unwrap();
      let first = last.chars().next().unwrap();
      if last.len() == 1 {
         return Ok(Key { modifiers, key: KeyCode::Char(first) });
      }

      Ok(match first {
         'F' | 'f' if last.len() > 1 => Key { key: KeyCode::F((&last[1..]).parse::<u8>().map_err(serde::de::Error::custom)?), modifiers },
         _ => {}
      })
   }
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Play {
    Context(Uri, Option<Offset>, Option<usize>),
}

impl Play {
    pub fn playlist(id: impl Id, offset: Option<usize>, position: Option<usize>) -> Self {
        Self::Context(
            Uri::new(Type::Playlist, id.id()),
            offset.map(Offset::Position),
            position,
        )
    }

    pub fn artist(id: impl Id, offset: Option<usize>, position: Option<usize>) -> Self {
        Self::Context(
            Uri::new(Type::Artist, id.id()),
            offset.map(Offset::Position),
            position,
        )
    }

    pub fn show(id: impl Id, offset: Option<usize>, position: Option<usize>) -> Self {
        Self::Context(
            Uri::new(Type::Show, id.id()),
            offset.map(Offset::Position),
            position,
        )
    }

    pub fn album(id: impl Id, offset: Option<usize>, position: Option<usize>) -> Self {
        Self::Context(
            Uri::new(Type::Album, id.id()),
            offset.map(Offset::Position),
            position,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Uri {
    ty: Type,
    id: String,
}

impl<I: Id> From<I> for Uri {
    fn from(value: I) -> Self {
        Self {
            ty: value._type(),
            id: value.id().to_string(),
        }
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let uri = match self.ty {
            Type::Collection => format!("spotify:user:{}:collection", self.id),
            Type::Collectionyourepisodes => {
                format!("spotify:user:{}:collection:your-episodes", self.id)
            }
            other => format!("spotify:{}:{}", other, self.id),
        };

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

    pub fn play_context_id<'a>(&'a self) -> Result<PlayContextId<'a>, Error> {
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Offset {
    Position(usize),
    Uri(String),
}

impl From<Offset> for rspotify::model::Offset {
    fn from(value: Offset) -> Self {
        match value {
            Offset::Uri(uri) => rspotify::model::Offset::Uri(uri),
            Offset::Position(pos) => {
                rspotify::model::Offset::Position(chrono::Duration::milliseconds(pos as i64))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rspotify::model::AlbumId;
    use std::collections::HashMap;

    use crate::keyevent;

    use super::{Action, ModalOpen, Play};

    #[test]
    fn serialize() {
        let mappings = HashMap::from([
            (keyevent!('r'), Action::Repeat),
            (
                keyevent!(F(1)),
                Action::Open(ModalOpen::Devices(Some(true))),
            ),
            (keyevent!(F(2)), Action::Open(ModalOpen::Devices(None))),
            (
                keyevent!(F(3)),
                Action::SetDevice("somedeviceid".to_string(), Some(true)),
            ),
            (
                keyevent!(F(4)),
                Action::SetDevice("somedeviceid".to_string(), None),
            ),
            (
                keyevent!(F(5)),
                Action::Play(Play::album(
                    AlbumId::from_id("3B69Rwb21o9LqQnJB9dw5O").unwrap(),
                    None,
                    None,
                )),
            ),
        ]);

        // let result = toml::to_string(&keyevent!(F(3))).unwrap();
    }
}
