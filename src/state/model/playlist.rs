use ratatui::{
    layout::Constraint,
    widgets::Cell,
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
    fn page_row(&self) -> Vec<(String, Option<Box<dyn Fn(String) -> Cell<'static>>>)> {
        vec![
            (self.name.clone(), None),
            (self.owner.display_name.clone().unwrap_or_default(), None),
            (if self.public.unwrap_or_default() {
                "public"
            } else {
                "private"
            }.to_string(), None),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        vec![
            Constraint::Fill(1),
            Constraint::Max(widths.get(1).copied().unwrap_or_default() as u16),
            Constraint::Max(widths.get(2).copied().unwrap_or_default() as u16),
        ]
    }
}

impl ActionList for Playlist {
    fn action_list(&self) -> Vec<(Key, Action)> {
        Vec::from([
            (key!(Enter), Action::Play(Play::playlist(self.id.clone(), None, None))),
            (key!('o'), Action::Open(Open::playlist(self))),
            // TODO: Favorite/Unfavorite
            // TODO: Open context
        ])
    }
}
