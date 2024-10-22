use chrono::Duration;
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
};
use rspotify::model::{EpisodeId, FullEpisode, Image, ResumePoint, SimplifiedEpisode};

use crate::state::{format_duration, window::PageRow};

#[derive(Debug, Clone, PartialEq)]
pub struct Episode {
    pub description: String,
    pub duration: Duration,
    pub explicit: bool,
    pub id: EpisodeId<'static>,
    pub images: Vec<Image>,
    pub is_playable: bool,
    pub name: String,
    pub resume_point: Option<ResumePoint>,
    pub publisher: Option<String>,
}

impl From<FullEpisode> for Episode {
    fn from(value: FullEpisode) -> Self {
        Self {
            description: value.description,
            duration: value.duration,
            explicit: value.explicit,
            id: value.id,
            images: value.images,
            is_playable: value.is_playable,
            name: value.name,
            resume_point: value.resume_point,
            publisher: Some(value.show.publisher),
        }
    }
}

impl From<SimplifiedEpisode> for Episode {
    fn from(value: SimplifiedEpisode) -> Self {
        Self {
            description: value.description,
            duration: value.duration,
            explicit: value.explicit,
            id: value.id,
            images: value.images,
            is_playable: value.is_playable,
            name: value.name,
            resume_point: value.resume_point,
            publisher: None,
        }
    }
}

impl PageRow for Episode {
    fn page_row(&self) -> Vec<(String, Style)> {
        vec![
            (format_duration(self.duration), Style::default()),
            (self.name.clone(), Style::default()),
            (if self.explicit { "explicit" } else { "" }.to_string(), Style::default().red()),
            (self.publisher.clone().unwrap_or_default(), Style::default()),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        widths.into_iter().map(|v| Constraint::Length(v as u16)).collect()
    }
}
