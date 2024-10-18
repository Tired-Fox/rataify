use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};
use rspotify::model::{AlbumId, AlbumType, Image, SavedAlbum, FullAlbum};

use crate::state::window::PageRow;

#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    pub artists: Vec<String>,
    pub album_type: AlbumType,
    pub genres: Vec<String>,
    pub id: AlbumId<'static>,
    pub images: Vec<Image>,
    pub name: String,
    pub popularity: u32,
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
            popularity: value.album.popularity,
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
            popularity: value.popularity,
        }
    }
}

impl PageRow for Album {
    type Container = Table<'static>;
    type Row = Row<'static>;

    fn page_row(&self) -> Self::Row {
        Row::new(vec![
            Cell::from(self.name.clone()),
            Cell::from(self.artists.join(", ")),
        ])
    }

    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State {
        TableState::default().with_selected(index)
    }

    fn page_widths<'a>(items: impl Iterator<Item = &'a Self>) -> Vec<Constraint>
    where
        Self: 'a,
    {
        vec![
            Constraint::Fill(1),
            Constraint::Length(
                items
                    .map(|a| a.artists.join(", ").len())
                    .max()
                    .unwrap_or_default() as u16,
            ),
        ]
    }

    fn page_container(items: Vec<Self::Row>, widths: Vec<Constraint>) -> Self::Container {
        Table::new(items, widths).highlight_style(Style::default().yellow())
    }
}
