use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Widget},
};
use tupy::{
    api::response::{Playback, PlaybackItem},
    DateTime, Duration, Local,
};

use crate::{Locked, Shared};

use super::format_duration;

pub struct UiPlayback {
    pub playback: Shared<Locked<Option<Playback>>>,
    pub last_playback_poll: Shared<Locked<DateTime<Local>>>,
}

impl Widget for UiPlayback {
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

        let playback = self.playback.lock().unwrap().clone();
        match playback.as_ref() {
            Some(playback) => {
                let mut progress = playback.progress.unwrap_or(Duration::zero());
                if playback.is_playing {
                    progress += Local::now() - *self.last_playback_poll.lock().unwrap();
                }

                match &playback.item {
                    PlaybackItem::Track(track) => {
                        let album = track.album.name.clone();
                        let artists = track
                            .artists
                            .iter()
                            .map(|a| a.name.as_str())
                            .collect::<Vec<&str>>()
                            .join(", ");

                        if let Some(device) = &playback.device {
                            let title = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints([
                                    Constraint::Percentage(60),
                                    Constraint::Percentage(40),
                                ])
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
                        let context = if let Some(show) = &episode.show {
                            show.publisher.as_deref().unwrap_or("").to_string()
                        } else {
                            String::new()
                        };

                        let title_layout = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                            .split(playing[0]);
                        Line::from(episode.name.clone())
                            .left_aligned()
                            .render(title_layout[0], buf);
                        Line::from(context)
                            .right_aligned()
                            .render(title_layout[1], buf);

                        if let Some(device) = &playback.device {
                            Line::from(device.name.clone()).render(playing[1], buf);
                        }

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
                            .render(playing[1], buf);
                    }
                    PlaybackItem::Unkown => {
                        Line::from("<Unknown Playback>")
                            .gray()
                            .centered()
                            .render(playing[1], buf);
                    }
                }
            }
            None => {
                Line::from("<No Playback>")
                    .red()
                    .centered()
                    .render(playing[1], buf);
            }
        }
    }
}

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
        // width - padding - time_progress - time_duration
        let rw = area.width as usize - 2 - time_prog.chars().count() - time_dur.chars().count();
        let scale = rw as f32 / self.total.num_milliseconds() as f32;
        let progress = (self.current.num_milliseconds() as f32 * scale) as u16;
        let remaining = rw as u16 - progress;
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
