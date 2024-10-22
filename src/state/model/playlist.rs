use std::collections::HashMap;

use ratatui::{
    layout::Constraint,
    style::Style,
};
use rspotify::model::{Image, PlaylistId, PublicUser, SimplifiedPlaylist};

use crate::{action::{Action, Open, Play}, input::Key, key, state::{window::PageRow, ActionList}};

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
    fn page_row(&self) -> Vec<(String, Style)> {
        vec![
            (self.name.clone(), Style::default()),
            (self.owner.display_name.clone().unwrap_or_default(), Style::default()),
            (if self.public.unwrap_or_default() {
                "public"
            } else {
                "private"
            }.to_string(), Style::default()),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}

impl ActionList for Playlist {
    fn action_list(&self) -> HashMap<Key, Action> {
        HashMap::from([
            (key!(Enter), Action::Play(Play::playlist(self.id.clone(), None, None))),
            (key!('o'), Action::Open(Open::playlist(self))),
            // TODO: Favorite/Unfavorite
            // TODO: Open context
        ])
    }
}
