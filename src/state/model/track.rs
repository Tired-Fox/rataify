use chrono::Duration;
use ratatui::{
    layout::Constraint,
    style::Stylize,
    text::Line,
    widgets::Cell,
};
use rspotify::model::{FullTrack, Image, SimplifiedTrack, TrackId};

use crate::{action::{Action, Play}, state::{format_duration, window::PageRow, ActionList}};

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub images: Vec<Image>,
    pub artists: Vec<String>,
    pub disc_number: i32,
    pub duration: Duration,
    pub explicit: bool,
    pub id: Option<TrackId<'static>>,
    pub name: String,
    pub track_number: u32,
}

impl From<FullTrack> for Track {
    fn from(value: FullTrack) -> Self {
        Self {
            images: value.album.images,
            artists: value.artists.into_iter().map(|a| a.name).collect(),
            disc_number: value.disc_number,
            duration: value.duration,
            explicit: value.explicit,
            id: value.id,
            name: value.name,
            track_number: value.track_number,
        }
    }
}

impl From<SimplifiedTrack> for Track {
    fn from(value: SimplifiedTrack) -> Self {
        Self {
            images: value.album.map(|v| v.images).unwrap_or_default(),
            artists: value.artists.into_iter().map(|a| a.name).collect(),
            disc_number: value.disc_number,
            duration: value.duration,
            explicit: value.explicit,
            id: value.id,
            name: value.name,
            track_number: value.track_number,
        }
    }
}

impl PageRow for Track {
    fn page_row(&self) -> Vec<(String, Option<Box<dyn Fn(String) -> Cell<'static>>>)> {
        vec![
            (
                format_duration(self.duration),
                Some(Box::new(|data| Cell::from(Line::from(data).right_aligned()).dark_gray())),
            ),
            (
                if self.explicit { "E" } else { "" }.to_string(),
                Some(Box::new(|data| Cell::from(data).red())),
            ),
            (self.name.clone(), None),
            (
                self.artists.join(", "),
                Some(Box::new(|data| {
                    Cell::from(data)
                        .magenta()
                })),
            ),
        ]
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

impl Track {
    pub fn play(&self) -> Action {
        Action::Play(Play::single(self.id.clone().unwrap().into(), None, None))
    }
}

impl ActionList for Track {
    fn action_list(&self, goto: bool) -> Vec<(crate::input::Key, crate::action::Action)> {
        self.action_list_with([], goto)
    }

    fn action_list_with(&self, initial: impl IntoIterator<Item=(crate::input::Key, Action)>, _goto: bool) -> Vec<(crate::input::Key, Action)> {
        initial.into_iter()
            .collect()
    }
}
