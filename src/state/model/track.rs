use chrono::Duration;
use ratatui::{layout::Constraint, style::{Style, Stylize}};
use rspotify::model::{FullTrack, Image, SimplifiedTrack, TrackId};

use crate::state::{format_duration, window::PageRow};



#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub images: Vec<Image>,
    pub artists: Vec<String>,
    pub disc_number: i32,
    pub duration: Duration,
    pub explicit: bool,
    pub id: Option<TrackId<'static>>,
    pub is_playable: Option<bool>,
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
            is_playable: value.is_playable,
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
            is_playable: value.is_playable,
            name: value.name,
            track_number: value.track_number,
        } 
    }
}

impl PageRow for Track {
    fn page_row(&self) -> Vec<(String, Style)> {
        vec![
            (format_duration(self.duration), Style::default()),
            (self.name.clone(), Style::default()),
            (if self.explicit { "explicit" } else { "" }.to_string(), Style::default().red()),
            (self.artists.join(", "), Style::default()),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}
