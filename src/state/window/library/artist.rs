use ratatui::{layout::Constraint, style::{Style, Stylize}, widgets::{Cell, Row, StatefulWidget, Table, TableState}};
use rspotify::model::{ArtistId, FullArtist, Image};

use crate::state::window::PageRow;


#[derive(Debug, Clone, PartialEq)]
pub struct Artist {
    pub followers: usize,
    pub genres: Vec<String>,
    pub id: ArtistId<'static>,
    pub images: Vec<Image>,
    pub name: String,
    pub popularity: u32,
}

impl From<FullArtist> for Artist {
    fn from(value: FullArtist) -> Self {
        Self {
            followers: value.followers.total as usize,
            genres: value.genres,
            id: value.id,
            images: value.images,
            name: value.name,
            popularity: value.popularity,
        } 
    }
}

impl PageRow for Artist {
    type Container = Table<'static>;
    type Row = Row<'static>;

    fn page_row(&self) -> Self::Row {
        Row::new(vec![
            Cell::from(self.name.clone()),
            Cell::from(self.genres.join(", ")),
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
                    .map(|i| i.genres.join(", ").len())
                    .max()
                    .unwrap_or_default() as u16,
            ),
        ]
    }

    fn page_container(items: Vec<Self::Row>, widths: Vec<Constraint>) -> Self::Container {
        Table::new(items, widths).highlight_style(Style::default().yellow())
    }
}
