use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};
use rspotify::model::{Image, PlaylistId, PublicUser, SimplifiedPlaylist};

use crate::state::window::PageRow;

#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
    pub collaborative: bool,
    pub id: PlaylistId<'static>,
    pub images: Vec<Image>,
    pub name: String,
    pub owner: PublicUser,
    pub public: Option<bool>,
    pub tracks: usize,
}

impl From<SimplifiedPlaylist> for Playlist {
    fn from(value: SimplifiedPlaylist) -> Self {
        Self {
            collaborative: value.collaborative,
            id: value.id,
            images: value.images,
            name: value.name,
            owner: value.owner,
            public: value.public,
            tracks: value.tracks.total as usize,
        }
    }
}

impl PageRow for Playlist {
    type Container = Table<'static>;
    type Row = Row<'static>;

    fn page_row(&self) -> Self::Row {
        Row::new(vec![
            Cell::from(self.name.clone()),
            Cell::from(self.owner.display_name.clone().unwrap_or_default()),
            Cell::from(if self.public.unwrap_or_default() {
                "public"
            } else {
                "private"
            }),
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
                    .map(|p| p.owner.display_name.as_deref().unwrap_or_default().len() as u16)
                    .max()
                    .unwrap_or(1)
                    .min(u16::MAX),
            ),
            Constraint::Length(7),
        ]
    }

    fn page_container(items: Vec<Self::Row>, widths: Vec<Constraint>) -> Self::Container {
        Table::new(items, widths).highlight_style(Style::default().yellow())
    }
}
