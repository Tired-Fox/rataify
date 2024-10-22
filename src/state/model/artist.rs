use std::collections::HashMap;

use ratatui::{layout::Constraint, style::Style};
use rspotify::model::{ArtistId, FullArtist, Image};

use crate::{action::{Action, Open, Play}, input::Key, key, state::{window::PageRow, ActionList}};


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
    fn page_row(&self) -> Vec<(String, Style)> {
        vec![
            (self.name.clone(), Style::default()),
            (self.genres.join(", "), Style::default()),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}

impl ActionList for Artist {
    fn action_list(&self) -> HashMap<Key, Action> {
        HashMap::from([
            (key!(Enter), Action::Play(Play::artist(self.id.clone(), None, None))),
            (key!('o'), Action::Open(Open::artist(self))),
            // TODO: Favorite/Unfavorite
            // TODO: Open context
        ])
    }
}
