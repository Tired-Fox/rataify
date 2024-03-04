use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Color;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Gauge, StatefulWidget, Widget};

use crate::state::State;
use crate::ui::Cover;

pub struct Footer;

impl StatefulWidget for Footer {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let padding = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(padding[1]);

        Cover.render(layout[0], buf, state);
        PlaybackInfo.render(layout[2], buf, state);
    }
}

pub struct PlaybackInfo;

impl StatefulWidget for PlaybackInfo {
    type State = State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(rect);

        NameArtist.render(layout[1], buf, state);
        Progress.render(layout[2], buf, state);
        PlayState.render(layout[3], buf, state);
    }
}

pub struct NameArtist;

impl StatefulWidget for NameArtist {
    type State = State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(rect);

        // ♡
        buf.set_string(
            layout[0].left(),
            layout[0].top(),
            &state.playback.name(),
            Style::default().bold(),
        );
        buf.set_string(
            layout[1].left(),
            layout[1].top(),
            &state.playback.artists().join(", "),
            Style::default().italic().dark_gray(),
        );
    }
}

pub struct Progress;

impl StatefulWidget for Progress {
    type State = State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let d = state.playback.duration();
        let p = state.playback.progress();
        let color = match state.playback.playing() {
            true => Color::Rgb(29, 185, 84),
            false => Color::Rgb(221, 136, 17),
        };

        Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().bold().fg(color).bg(Color::Rgb(25, 20, 20)))
            .label(format!(
                "{}:{:0>2} / {}:{:0>2}",
                p.num_minutes(),
                p.num_seconds() % 60,
                d.num_minutes(),
                d.num_seconds() % 60
            ))
            .ratio(state.playback.percent())
            .render(rect, buf);
    }
}

pub struct PlayState;
impl StatefulWidget for PlayState {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(9),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(area);

        let shuffle_style = match state.playback.shuffle() {
            true => Style::default().bold(),
            false => Style::default().italic().fg(Color::DarkGray),
        };

        buf.set_string(layout[0].left(), layout[0].top(), if state.playback.liked() { "♥" } else { " " }, Style::default());
        buf.set_string(layout[1].left(), layout[1].top(), "Shuffle ", shuffle_style);
        buf.set_string(
            layout[2].left(),
            layout[2].top(),
            format!("Repeat: {}", state.playback.repeat()),
            Style::default(),
        );
        buf.set_string(layout[3].left(), layout[3].top(), "?", Style::default());
    }
}
