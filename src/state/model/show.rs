use std::collections::HashMap;

use ratatui::{
    layout::Constraint,
    style::Style,
};
use rspotify::model::{Image, Show as SpotifyShow, ShowId, SimplifiedShow};

use crate::{action::{Action, Open, Play}, input::Key, key, state::{window::PageRow, ActionList}};

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
    fn page_row(&self) -> Vec<(String, Style)> {
        vec![
            (self.name.clone(), Style::default()),
            (if self.explicit { "explicit" } else { "" }.to_string(), Style::default()),
            (self.publisher.clone(), Style::default()),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}

impl ActionList for Show {
    fn action_list(&self) -> HashMap<Key, Action> {
        HashMap::from([
            (key!(Enter), Action::Play(Play::show(self.id.clone(), None, None))),
            (key!('o'), Action::Open(Open::show(self))),
            // TODO: Favorite/Unfavorite
            // TODO: Open context
        ])
    }
}
