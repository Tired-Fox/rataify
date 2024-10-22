use std::collections::HashMap;

use ratatui::{
    layout::Constraint,
    style::Style,
};
use rspotify::model::{AlbumId, AlbumType, FullAlbum, Image, SavedAlbum, SimplifiedAlbum};

use crate::{action::{Action, Open, Play}, input::Key, key, state::{window::PageRow, ActionList}};

#[derive(Debug, Clone, PartialEq)]
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
    fn page_row(&self) -> Vec<(String, Style)> {
        vec![
            (self.name.clone(), Style::default()),
            (self.artists.join(", "), Style::default()),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}

impl ActionList for Album {
    fn action_list(&self) -> HashMap<Key, Action> {
        HashMap::from([
            (key!(Enter), Action::Play(Play::album(self.id.clone(), None, None))),
            (key!('o'), Action::Open(Open::album(self))),
            // TODO: Favorite/Unfavorite
            // TODO: Open context
        ])
    }
}
