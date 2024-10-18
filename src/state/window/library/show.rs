use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};
use rspotify::model::{Image, Show as SpotifyShow, ShowId, SimplifiedShow};

use crate::state::window::PageRow;

#[derive(Debug, Clone, PartialEq)]
pub struct Show {
    pub description: String,
    pub explicit: bool,
    pub id: ShowId<'static>,
    pub images: Vec<Image>,
    pub languages: Vec<String>,
    pub name: String,
    pub publisher: String,
}

impl From<SpotifyShow> for Show {
    fn from(value: SpotifyShow) -> Self {
        Self {
            description: value.show.description,
            explicit: value.show.explicit,
            id: value.show.id,
            images: value.show.images,
            languages: value.show.languages,
            name: value.show.name,
            publisher: value.show.publisher,
        }
    }
}

impl From<SimplifiedShow> for Show {
    fn from(value: SimplifiedShow) -> Self {
        Self {
            description: value.description,
            explicit: value.explicit,
            id: value.id,
            images: value.images,
            languages: value.languages,
            name: value.name,
            publisher: value.publisher,
        }
    }
}

impl PageRow for Show {
    type Container = Table<'static>;
    type Row = Row<'static>;

    fn page_row(&self) -> Self::Row {
        Row::new(vec![
            Cell::from(self.name.clone()),
            Cell::from(if self.explicit { "explicit" } else { "" }),
            Cell::from(self.publisher.clone()),
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
            Constraint::Length(9),
            Constraint::Length(
                items
                    .map(|s| s.publisher.len())
                    .max()
                    .unwrap_or_default() as u16,
            ),
        ]
    }

    fn page_container(items: Vec<Self::Row>, widths: Vec<Constraint>) -> Self::Container {
        Table::new(items, widths).highlight_style(Style::default().yellow())
    }
}
