use ratatui::{
    layout::Constraint,
    widgets::Cell,
};
use rspotify::model::{PlayableItem, PlaylistItem};

use crate::state::window::PageRow;

use super::{Episode, Track};

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Track(Track),
    Episode(Episode)
}

impl From<PlaylistItem> for Item {
    fn from(value: PlaylistItem) -> Self {
        value.track.unwrap().into()
    }
}

impl From<PlayableItem> for Item {
    fn from(value: PlayableItem) -> Self {
        match value {
            PlayableItem::Track(track) => Self::Track(track.into()),
            PlayableItem::Episode(ep) => Self::Episode(ep.into()),
        }
    }
}

impl PageRow for Item {
    fn page_row(&self) -> Vec<(String, Option<Box<dyn Fn(String) -> Cell<'static>>>)> {
        match self {
            Self::Track(track) => track.page_row(),
            Self::Episode(ep) => ep.page_row(),
        }
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        vec![
            Constraint::Length(widths.first().copied().unwrap_or_default() as u16),
            Constraint::Length(1),
            Constraint::Ratio(3, 4),
            Constraint::Ratio(1, 4),
        ]
    }
}
