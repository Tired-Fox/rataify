use std::fs::OpenOptions;
use std::io::Write;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Color;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Gauge, StatefulWidget, Widget};

use crate::state::State;

pub struct Footer;

impl StatefulWidget for Footer {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let footer = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(11),
                Constraint::Length(1),
                Constraint::Min(5),
            ])
            .split(area);

        Cover.render(footer[0], buf, state);
        PlaybackInfo.render(footer[2], buf, state);
    }
}

pub struct Cover;

// TODO: Move cover generation to this render method
impl StatefulWidget for Cover {
    type State = State;

    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let width = rect.width - 2;
        let height = rect.height - 2;
        let cover = &state.playback.cover;

        let start_y = ((cover.len() - 1) - height as usize) / 2;
        let start_x = ((cover.get(0).unwrap().len() - 1) - width as usize) / 2;

        for y in 0..height {
            for x in 0..width {
                buf.get_mut(rect.left() + x + 1, rect.top() + y + 1)
                    .set_symbol(&cover.get(start_y + y as usize).unwrap().get(start_x + x as usize).unwrap().to_string());
            }
        }

        buf.get_mut(rect.left(), rect.top()).set_symbol("┌─");
        buf.get_mut(rect.right() - 2, rect.top()).set_symbol("─┐");
        buf.get_mut(rect.left(), rect.bottom() - 1).set_symbol("└─");
        buf.get_mut(rect.right() - 2, rect.bottom() - 1).set_symbol("─┘");
    }
}

pub struct PlaybackInfo;

impl StatefulWidget for PlaybackInfo {
    type State = State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let playback_info = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(rect);

        NameArtist.render(playback_info[1], buf, state);
        Progress.render(playback_info[2], buf, state);
    }
}

pub struct NameArtist;

impl StatefulWidget for NameArtist {
    type State = State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let name_artist = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(rect);

        buf.set_string(name_artist[0].left(), name_artist[0].top(), &state.playback.name(), Style::default().bold());
        buf.set_string(name_artist[1].left(), name_artist[1].top(), &state.playback.artists().join(", "), Style::default().italic().dark_gray());
    }
}

pub struct Progress;

impl StatefulWidget for Progress {
    type State = State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let p = state.playback.progress();
        let d = state.playback.duration();
        let color = match state.playback.playing() {
            true => Color::Rgb(29, 185, 84),
            false => Color::Rgb(221, 136, 17),
        };

        Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().italic().bold().fg(color).bg(Color::Rgb(25, 20, 20)))
            .label(format!("{}:{:0>2} / {}:{:0>2}", p.num_minutes(), p.num_seconds() % 60, d.num_minutes(), d.num_seconds() % 60))
            .ratio(state.playback.percent())
            .render(rect, buf);
    }
}
