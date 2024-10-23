use chrono::Duration;
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    text::Line,
    widgets::Cell,
};
use rspotify::model::{EpisodeId, FullEpisode, Image, ResumePoint, SimplifiedEpisode};

use crate::{action::{Action, Play}, state::{format_duration, window::PageRow, ActionList}};

#[derive(Debug, Clone, PartialEq)]
pub struct Episode {
    pub description: String,
    pub duration: Duration,
    pub explicit: bool,
    pub id: EpisodeId<'static>,
    pub images: Vec<Image>,
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
            name: value.name,
            resume_point: value.resume_point,
            publisher: None,
        }
    }
}

impl PageRow for Episode {
    fn page_row(&self) -> Vec<(String, Option<Box<dyn Fn(String) -> Cell<'static>>>)> {
        let finished = self
            .resume_point
            .as_ref()
            .map(|rp| rp.fully_played)
            .unwrap_or_default();
        vec![
            (
                format_duration(self.duration),
                Some(Box::new(move |data| {
                    Cell::from(Line::from(data).right_aligned()).style(if finished {
                        Style::default().green()
                    } else {
                        Style::default().dark_gray()
                    })
                })),
            ),
            (
                if self.explicit { "E" } else { "" }.to_string(),
                Some(Box::new(|data| Cell::from(data).red())),
            ),
            (self.name.clone(), None),
            (
                self.publisher.clone().unwrap_or_default(),
                Some(Box::new(|data| {
                    Cell::from(Line::from(data).right_aligned())
                        .magenta()
                })),
            ),
        ]
    }

    fn page_widths(widths: Vec<usize>) -> Vec<Constraint> {
        vec![
            Constraint::Length(widths.first().copied().unwrap_or_default() as u16),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(widths.get(3).copied().unwrap_or_default() as u16),
        ]
    }
}

impl Episode {
    pub fn play(&self) -> Action {
        Action::Play(Play::single(self.id.clone().into(), None, None))
    }
}

impl ActionList for Episode {
    fn action_list(&self, goto: bool) -> Vec<(crate::input::Key, crate::action::Action)> {
        self.action_list_with([], goto)
    }

    fn action_list_with(&self, initial: impl IntoIterator<Item=(crate::input::Key, Action)>, _goto: bool) -> Vec<(crate::input::Key, Action)> {
        initial.into_iter()
            .collect()
    }
}
