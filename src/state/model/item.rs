use ratatui::{
    layout::Constraint,
    style::Style,
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
    fn page_row(&self) -> Vec<(String, Style)> {
        match self {
            Self::Track(track) => track.page_row(),
            Self::Episode(ep) => ep.page_row(),
        }
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}
