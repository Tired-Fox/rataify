use std::collections::HashMap;

use ratatui::{
    layout::Constraint,
    style::{Style, Stylize}, widgets::Cell,
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
    fn page_row(&self) -> Vec<(String, Option<Box<dyn Fn(String) -> Cell<'static>>>)> {
        let explicit = self.explicit;
        vec![
            (self.name.clone(), None),
            (if explicit { "E" } else { "" }.to_string(), Some(Box::new(move |data| Cell::from(data).red()))),
            (self.publisher.clone(), None),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        vec![
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(widths.get(2).copied().unwrap_or_default() as u16),
        ]
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
