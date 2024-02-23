use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph};
use crate::state::State;

pub fn counter(state: &State, frame: &mut Frame) {
    frame.render_widget(Paragraph::new(format!("Counter: {}", state.counter)), frame.size());
}


pub fn mock_player(state: &State, frame: &mut Frame) {
    let main= Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3)
        ])
        .split(frame.size());

    let main_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Max(40),
            Constraint::Min(1)
        ])
        .split(main[0]);

    match state.now_playing {
        Some(ref now_playing) => {
            let np_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Percentage(90),
                    Constraint::Percentage(10),
                    Constraint::Length(1),
                ])
                .split(main_center[1]);
            match now_playing.cover {
                Some(ref cover) => {
                    cover.render(frame, centered_rect(np_layout[1], 26, 13));
                },
                None => {}
            }
            frame.render_widget(
                Paragraph::new(now_playing.name.clone())
                    .alignment(Alignment::Center)
                    .style(Style::default().bold()),
                np_layout[2]
            );
        }
        None => {
            // Help message to get user going with starting music
            frame.render_widget(Paragraph::new("Main Center").alignment(Alignment::Center), main_center[1]);
        }

    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    frame.render_widget(
        Paragraph::new("Progress")
            .alignment(Alignment::Center)
            .block(block),
        main[1]
    );
}

fn centered_rect(r: Rect, width: u16, height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(height),
            Constraint::Min(1),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(width),
            Constraint::Min(1),
        ])
        .split(popup_layout[1])[1]
}