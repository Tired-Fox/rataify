use crate::state::State;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

pub fn counter(state: &State, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new(format!("Counter: {}", state.counter)),
        frame.size(),
    );
}

pub fn mock_player(state: &State, frame: &mut Frame) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.size());

    let main_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Max(40), Constraint::Min(1)])
        .split(main[0]);

    match state.now_playing {
        Some(ref now_playing) => {
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
                Paragraph::new(now_playing.cover(14)),
                centered_rect(now_playing_layout[0], (14.0 * 2.5) as u16),
            );

            frame.render_widget(
                Paragraph::new(now_playing.name.clone())
                    .style(Style::default().bold())
                    .alignment(Alignment::Center),
                now_playing_layout[2],
            );
            frame.render_widget(
                Paragraph::new(now_playing.artist.clone())
                    .style(Style::default().italic().dark_gray())
                    .alignment(Alignment::Center),
                now_playing_layout[3],
            );
        }
        None => {
            // Help message to get user going with starting music
            frame.render_widget(
                Paragraph::new("Main Center").alignment(Alignment::Center),
                main_center[1],
            );
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    frame.render_widget(
        Paragraph::new("Progress")
            .alignment(Alignment::Center)
            .block(block),
        main[1],
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

