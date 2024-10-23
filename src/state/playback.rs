use chrono::{DateTime, Duration, Local};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Widget},
};
use rspotify::model::{
    Context, CurrentPlaybackContext, CurrentlyPlayingType, Device, PlayableItem, RepeatState
};

#[derive(Debug, Clone)]
pub struct Playback {
    pub timestamp: DateTime<Local>,

    pub device: Device,
    pub repeat_state: RepeatState,
    pub shuffle_state: bool,
    pub context: Option<Context>,
    pub progress: Option<Duration>,
    pub is_playing: bool,
    pub item: Option<PlayableItem>,
    pub currently_playing_type: CurrentlyPlayingType,
}

impl From<CurrentPlaybackContext> for Playback {
    fn from(value: CurrentPlaybackContext) -> Self {
        Self {
            timestamp: Local::now(),

            device: value.device,
            repeat_state: value.repeat_state,
            shuffle_state: value.shuffle_state,
            context: value.context,
            progress: value.progress,
            is_playing: value.is_playing,
            item: value.item,
            currently_playing_type: value.currently_playing_type,
        }
    }
}

impl Widget for &Playback {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default().padding(Padding::horizontal(1));
        let lines = Layout::vertical([Constraint::Length(1); 3]).split(block.inner(area));

        let (title, by, duration) = match self.item.as_ref() {
            Some(PlayableItem::Track(track)) => (
                track.name.clone(),
                track
                    .artists
                    .iter()
                    .map(|v| v.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                track.duration,
            ),
            Some(PlayableItem::Episode(episode)) => (
                episode.name.clone(),
                episode.show.publisher.clone(),
                episode.duration,
            ),
            None => (
                format!("<{:?}>", self.currently_playing_type),
                String::new(),
                Duration::default(),
            ),
        };

        let title_line =
            Layout::horizontal([Constraint::Ratio(3, 1), Constraint::Ratio(1, 3)]).split(lines[0]);
        Line::from(title).bold().render(title_line[0], buf);
        Line::from(by)
            .magenta()
            .italic()
            .right_aligned()
            .render(title_line[1], buf);

        let info_line =
            Layout::horizontal([Constraint::Ratio(3, 1), Constraint::Ratio(1, 3)]).split(lines[2]);
        Line::from(vec![
            Span::from("Shuffle ").dark_gray(),
            Span::from(self.shuffle_state.to_string()).fg(if self.shuffle_state { Color::Green } else { Color::Red }),
            Span::from(" Repeat ").dark_gray(),
            Span::from(format!("{:?}", self.repeat_state)).fg(match self.repeat_state {
                RepeatState::Off => Color::Red,
                RepeatState::Context => Color::Magenta,
                RepeatState::Track => Color::Cyan,
            }),
        ])
            .render(info_line[0], buf);

        Line::from(
            format!("{} {}",
                self.device.name,
                self.device
                .volume_percent
                .as_ref()
                .map(|v| format!("{v} %"))
                .unwrap_or_default(),
            )
        )
        .dark_gray()
        .right_aligned()
        .render(info_line[1], buf);

        let progress = if self.is_playing {
            (self.progress.unwrap_or_default() + (Local::now() - self.timestamp)).min(duration)
        } else {
            self.progress.unwrap_or_default().min(duration)
        };

        let (dtag, ptag) = if duration >= Duration::hours(1) {
            (
                format!(
                    "{:02}:{:02}:{:02}",
                    duration.num_hours() % 24,
                    duration.num_minutes() % 60,
                    duration.num_seconds() % 60
                ),
                format!(
                    "{:02}:{:02}:{:02}",
                    progress.num_hours() % 24,
                    progress.num_minutes() % 60,
                    progress.num_seconds() % 60
                ),
            )
        } else {
            (
                format!(
                    "{:02}:{:02}",
                    duration.num_minutes() % 60,
                    duration.num_seconds() % 60
                ),
                format!(
                    "{:02}:{:02}",
                    progress.num_minutes() % 60,
                    progress.num_seconds() % 60
                ),
            )
        };

        let ratio = if progress.num_milliseconds() == 0 || duration.num_milliseconds() == 0 {
            0.0
        } else {
            progress.num_milliseconds() as f64 / duration.num_milliseconds() as f64
        };

        let full_width = lines[2].width;
        let bar_width = full_width - ((dtag.len() as u16 + 1) * 2);
        let filled = (bar_width as f64 * ratio) as u16;

        let (time_style, bar_style) = if self.is_playing {
            (Style::default(), Style::default().green())
        } else {
            (Style::default().dark_gray(), Style::default())
        };

        Line::from(vec![
            Span::from(format!("{ptag} ")).style(time_style),
            Span::from((0..filled).map(|_| '─').collect::<String>()).style(bar_style),
            Span::from((0..bar_width - filled).map(|_| '─').collect::<String>()).dark_gray(),
            Span::from(format!(" {dtag}")).style(time_style),
        ])
        .render(lines[1], buf);
    }
}
