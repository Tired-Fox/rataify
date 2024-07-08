use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, style::{Stylize, Style}, text::{Line, Span}, widgets::{Block, Borders, StatefulWidget, Widget}
};
use tupy::{api::response::{Device, PlaybackItem, Repeat}, DateTime, Duration, Local};

use crate::state::playback::{Playback, PlaybackState};

use super::{format_duration, COLORS};

impl Widget for &PlaybackState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.playback.as_ref() {
            Some(playback) => StatefulWidget::render(
                playback,
                area,
                buf,
                &mut self.last_playback_poll.clone(),
            ),
            None => {
                NoPlayback.render(area, buf);
            }
        }
    }
}

pub struct NoPlayback;
impl Widget for NoPlayback {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        UI {
            title: Span::from("<No Playback>"),
            ..Default::default()
        }
            .render(area, buf);
    }
}

impl StatefulWidget for &Playback {
    type State = DateTime<Local>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let last_poll = *state;

        let mut progress = self.progress.unwrap_or(Duration::zero());
        if self.is_playing {
            progress += Local::now() - last_poll;
        }

        match &self.item {
            PlaybackItem::Track(track) => {
                //let album = track.album.name.clone();
                let artists = track
                    .artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ");

                UI {
                    title: Span::from(track.name.clone()),
                    saved: self.saved,
                    device: self.device.clone(),
                    shuffle: Some(self.shuffle),
                    repeat: Some(self.repeat),
                    context: Some(Line::from(artists).right_aligned()),
                    progress: Some(progress),
                    duration: Some(track.duration),
                    ..Default::default()
                }
                    .render(area, buf);
            }
            PlaybackItem::Episode(episode) => {
                let name = episode.show.as_ref().map(|s| Line::from(s.name.clone()).right_aligned());

                UI {
                    title: Span::from(episode.name.clone()),
                    saved: self.saved,
                    device: self.device.clone(),
                    shuffle: Some(self.shuffle),
                    repeat: Some(self.repeat),
                    context: name,
                    progress: Some(progress),
                    duration: Some(episode.duration),
                    ..Default::default()
                }
                    .render(area, buf);
            }
            PlaybackItem::Ad => {
                UI {
                    title: Span::from("<Advertisement>"),
                    ..Default::default()
                }
                    .render(area, buf);
            }
            PlaybackItem::Unkown => {
                UI {
                    title: Span::from("<Unknown Playback>"),
                    ..Default::default()
                }
                    .render(area, buf);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProgressBar {
    current: Duration,
    total: Duration,
}

impl Widget for ProgressBar {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.current > self.total {
            self.current = self.total;
        }

        let time_prog = format_duration(self.current);
        let time_dur = format_duration(self.total);
        let rw = area.width as usize - 2 - time_prog.chars().count() - time_dur.chars().count();

        let (progress, remaining) = if self.total != Duration::zero() {
            let scale = rw as f32 / self.total.num_milliseconds() as f32;
            let progress = (self.current.num_milliseconds() as f32 * scale) as u16;
            (progress, rw as u16 - progress)
        } else {
            (0, rw as u16)
        };

        Line::from(vec![
            time_prog.bold(),
            " ".into(),
            (0..progress)
                .map(|_| "─")
                .collect::<String>()
                .green()
                .bold(),
            (0..remaining).map(|_| "┄").collect::<String>().black(),
            " ".into(),
            time_dur.bold(),
        ])
        .render(area, buf);
    }
}

#[derive(Debug, Clone, Default)]
struct UI<'a> {
    title: Span<'a>,
    fully_played: bool,
    saved: bool,
    context: Option<Line<'a>>,

    device: Option<Device>,
    shuffle: Option<bool>,
    repeat: Option<Repeat>,

    progress: Option<Duration>,
    duration: Option<Duration>,
}

impl<'a> Widget for UI<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {

        let block = Block::bordered().borders(Borders::LEFT | Borders::RIGHT);
        let play_area = block.inner(area);
        let playing = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(play_area);
        
        let info_layout = Layout::horizontal([
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Length(9),
            Constraint::Fill(1),
        ])
            .split(playing[1]);

        if let Some(shuffle) = self.shuffle {
            Line::from(vec![
                Span::from("Shuffle: "),
                Span::from(if shuffle { "On" } else { "Off" }).style(if shuffle {
                    Style::default().green()
                } else {
                    Style::default().red()
                })
            ])
                .dim()
                .render(info_layout[0], buf);
        }
        if let Some(repeat) = self.repeat {
            Line::from(vec![
                Span::from("Repeat: "),
                Span::from(format!("{:?}", repeat)).style(match repeat {
                    Repeat::Off => Style::default().red(),
                    Repeat::Track => Style::default().cyan(),
                    Repeat::Context => Style::default().yellow(),
                })
            ])
                .dim()
                .render(info_layout[1], buf);
        }
        if let Some(device) = &self.device {
            if !device.is_restricted && device.supports_volume {
                Line::from(vec![
                    Span::from("Vol: "),
                    Span::from(format!("{}%", device.volume_percent)).magenta()
                ])
                    .dim()
                    .render(info_layout[2], buf);
            }
            Line::from(vec!["Playing on ".into(), device.name.clone().magenta()])
                .right_aligned()
                .dim()
                .gray()
                .render(info_layout[3], buf);
        }

        let title = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(2), Constraint::Fill(1)])
            .split(playing[0]);

        render_title(title[0], buf, self.title, self.fully_played);
        Line::from(if self.saved { "♥ " } else { "  " }).style(COLORS.like).render(title[1], buf);
        if let Some(c) = &self.context {
            c.render(title[2], buf);
        }

        ProgressBar {
            current: self.progress.unwrap_or(Duration::zero()),
            total: self.duration.unwrap_or(Duration::zero()),
        }
        .render(playing[2], buf);
    }
}

fn render_title<'a>(area: Rect, buf: &mut Buffer, title: Span<'a>, fully_played: bool) {
    if fully_played {
        Line::from(vec![
            title,
            Span::from(" ✓").style(COLORS.finished),
        ]).render(area, buf);
    } else {
        title.render(area, buf);
    }
}
