use ratatui::{
    layout::Constraint, style::Stylize, widgets::Cell
};
use rspotify::model::{AlbumId, AlbumType, FullAlbum, Image, SavedAlbum, SimplifiedAlbum};
use serde::{Deserialize, Serialize};

use crate::{action::{Action, Offset, Open, Play}, input::Key, key, state::{window::PageRow, ActionList}};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Album {
    pub artists: Vec<String>,
    pub album_type: AlbumType,
    pub genres: Vec<String>,
    pub id: AlbumId<'static>,
    pub images: Vec<Image>,
    pub name: String,
}

impl From<SimplifiedAlbum> for Album {
    fn from(value: SimplifiedAlbum) -> Self {
        Self {
            artists: value.artists.into_iter().map(|v| v.name).collect(),
            album_type: match value.album_type.as_deref() {
               Some("single") => AlbumType::Single,
               Some("appears_on") => AlbumType::AppearsOn,
               Some("compilation") => AlbumType::Compilation,
               _ => AlbumType::Album,
            },
            genres: Vec::default(),
            id: value.id.unwrap(),
            images: value.images,
            name: value.name,
        }
    }
}

impl From<SavedAlbum> for Album {
    fn from(value: SavedAlbum) -> Self {
        Self {
            artists: value.album.artists.into_iter().map(|v| v.name).collect(),
            album_type: value.album.album_type,
            genres: value.album.genres,
            id: value.album.id,
            images: value.album.images,
            name: value.album.name,
        }
    }
}

impl From<FullAlbum> for Album {
    fn from(value: FullAlbum) -> Self {
        Self {
            artists: value.artists.into_iter().map(|v| v.name).collect(),
            album_type: value.album_type,
            genres: value.genres,
            id: value.id,
            images: value.images,
            name: value.name,
        }
    }
}

impl PageRow for Album {
    fn page_row(&self) -> Vec<(String, Option<Box<dyn Fn(String) -> Cell<'static>>>)> {
        vec![
            (self.name.clone(), None),
            (self.artists.join(", "), Some(Box::new(|data| Cell::from(data).magenta()))),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        vec![
            Constraint::Fill(1),
            Constraint::Length(widths.get(1).copied().unwrap_or_default() as u16)
        ]
    }
}

impl Album {
    pub fn play(&self, offset: Option<Offset>) -> Action {
        Action::Play(Play::album(self.id.clone(), offset, None))
    }
}

impl ActionList for Album {
    fn action_list(&self, goto: bool) -> Vec<(Key, Action)> {
        self.action_list_with([], goto)
    }

    fn action_list_with(&self, initial: impl IntoIterator<Item=(Key, Action)>, goto: bool) -> Vec<(Key, Action)> {
        let mut maps = initial.into_iter()
            .chain([
                (key!(Enter), self.play(None))
            ])
            .collect::<Vec<_>>();

        if goto {
            maps.push((key!('o'), Action::Open(Open::album(self))))
        }

        // TODO: Favorite/Unfavorite
        // TODO: Open context
        
        maps
    }
}
