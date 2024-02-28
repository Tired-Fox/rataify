use crate::state::State;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};

use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Gauge};
use ratatui::Frame;
use ratatui::text::Span;

pub fn counter(state: &State, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new(format!("Counter: {}", state.counter)),
        frame.size(),
    );
}

pub fn mock_player(state: &State, frame: &mut Frame) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(5)])
        .split(frame.size());

    frame.render_widget(
        Paragraph::new("").block(Block::default().borders(Borders::ALL)),
        main[0]
    );
    let main_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Max(40), Constraint::Min(1)])
        .split(main[0]);

    let npl = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(17),
            Constraint::Min(1),
        ])
        .split(main_center[1]);

    let now_playing_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(npl[1]);

    frame.render_widget(
        Paragraph::new(state.playback.cover(14)),
        centered_rect(now_playing_layout[0], (14.0 * 2.5) as u16),
    );

    frame.render_widget(
        Paragraph::new(state.playback.name())
            .style(Style::default().bold())
            .alignment(Alignment::Center),
        now_playing_layout[2],
    );
    frame.render_widget(
        Paragraph::new(state.playback.artists().join(", "))
            .style(Style::default().italic().dark_gray())
            .alignment(Alignment::Center),
        now_playing_layout[3],
    );

    let footer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(1)])
        .split(main[1]);

    frame.render_widget(
        Paragraph::new(state.playback.cover(5)),
        footer[0],
    );

    let footer_details = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(footer[1]);

    frame.render_widget(Paragraph::new(state.playback.name()).style(Style::default().bold()), footer_details[1]);
    frame.render_widget(Paragraph::new(state.playback.artists().join(", ")).style(Style::default().italic().dark_gray()), footer_details[2]);

    let p = state.playback.progress();
    let d = state.playback.duration();
    let progress = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Rgb(29, 185, 84)).bg(Color::Rgb(25, 20, 20)))
        .label(format!("{}:{:0>2} / {}:{:0>2}", p.num_minutes(), p.num_seconds() % 60, d.num_minutes(), d.num_seconds() % 60))
        .ratio(state.playback.percent());
    frame.render_widget(
        progress,
        footer_details[3],
    );
}

fn centered_rect(r: Rect, width: u16) -> Rect {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(width),
            Constraint::Min(1),
        ])
        .split(r)[1]
}

