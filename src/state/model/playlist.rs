use ratatui::{
    layout::Constraint, style::Stylize, widgets::Cell
};
use rspotify::model::{FullPlaylist, Image, PlaylistId, PublicUser, SimplifiedPlaylist};
use serde::{Deserialize, Serialize};

use crate::{action::{Action, Offset, Open, Play}, input::Key, key, state::{window::PageRow, ActionList}};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

impl From<FullPlaylist> for Playlist {
    fn from(value: FullPlaylist) -> Self {
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
            }.to_string(), Some(Box::new(|data| Cell::from(data).cyan()))),
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

impl Playlist {
    pub fn play(&self, offset: Option<Offset>) -> Action {
        Action::Play(Play::playlist(self.id.clone(), offset, None))
    }
}

impl ActionList for Playlist {
    fn action_list(&self, goto: bool) -> Vec<(Key, Action)> {
        self.action_list_with([], goto)
    }

    fn action_list_with(&self, initial: impl IntoIterator<Item=(Key, Action)>, goto: bool) -> Vec<(Key, Action)> {
        let mut maps: Vec<_> = initial.into_iter()
            .chain([
                (key!(Enter), self.play(None))
            ])
            .collect();

        if goto {
            maps.push((key!('o'), Action::Open(Open::playlist(self))));
        }

        // TODO: Favorite/Unfavorite
        // TODO: Open context

        maps
    }
}
