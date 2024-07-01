use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Widget},
};
use tupy::{api::response::PlaybackItem, DateTime, Duration, Local};

use crate::state::Playback;

use super::format_duration;

pub struct NoPlayback;
impl Widget for NoPlayback {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default().borders(Borders::all());
        let play_area = block.inner(area);
        let playing = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(play_area);

        Line::from("<No Playback>")
            .red()
            .render(playing[0], buf);
        ProgressBar::default().render(playing[2], buf);
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
        let block = Block::default().borders(Borders::all());
        let play_area = block.inner(area);
        let playing = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(play_area);

        let mut progress = self.progress.unwrap_or(Duration::zero());
        if self.is_playing {
            progress += Local::now() - *state;
        }

        match &self.item {
            PlaybackItem::Track(track) => {
                let album = track.album.name.clone();
                let artists = track
                    .artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ");

                if let Some(device) = &self.device {
                    let title = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                        .split(playing[0]);
                    Line::from(track.name.clone()).render(title[0], buf);
                    Line::from(vec!["Playing on ".into(), device.name.clone().magenta()])
                        .right_aligned()
                        .dim()
                        .gray()
                        .render(title[1], buf);
                } else {
                    Line::from(track.name.clone()).render(playing[0], buf);
                }

                let context = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(playing[1]);

                Line::from(artists).render(context[0], buf);

                Line::from(album).right_aligned().render(context[1], buf);

                ProgressBar {
                    current: progress,
                    total: track.duration,
                }
                .render(playing[2], buf);
            }
            PlaybackItem::Episode(episode) => {
                let (name, publisher) = if let Some(show) = &episode.show {
                    (
                        show.name.clone(),
                        show.publisher.as_deref().unwrap_or("").to_string(),
                    )
                } else {
                    (String::new(), String::new())
                };

                if let Some(device) = &self.device {
                    let title = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                        .split(playing[0]);
                    Line::from(episode.name.clone()).render(title[0], buf);
                    Line::from(vec!["Playing on ".into(), device.name.clone().magenta()])
                        .right_aligned()
                        .dim()
                        .gray()
                        .render(title[1], buf);
                } else {
                    Line::from(episode.name.clone()).render(playing[0], buf);
                }

                let context = Layout::horizontal([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                    .split(playing[1]);
                        
                Line::from(name)
                    .render(context[0], buf);
                Line::from(publisher)
                    .right_aligned()
                    .render(context[1], buf);

                ProgressBar {
                    current: progress,
                    total: episode.duration,
                }
                .render(playing[2], buf);
            }
            PlaybackItem::Ad => {
                Line::from("<Advertisement>")
                    .yellow()
                    .centered()
                    .render(playing[0], buf);
                ProgressBar::default().render(playing[2], buf);
            }
            PlaybackItem::Unkown => {
                Line::from("<Unknown Playback>")
                    .gray()
                    .centered()
                    .render(playing[1], buf);
                ProgressBar::default().render(playing[2], buf);
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
