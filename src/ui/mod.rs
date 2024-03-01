use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::state::{MainWindow, ModalWindow, State, WindowState};
use crate::ui::footer::{Cover, Footer};
use crate::ui::modal::DeviceSelect;

pub mod icon;
mod footer;
mod header;
mod modal;

pub fn counter(state: &State, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new(format!("Counter: {}", state.counter)),
        frame.size(),
    );
}

pub fn mock_player(state: &mut State, frame: &mut Frame) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(5)
        ])
        .split(frame.size());

    // HEADER
    // frame.render_stateful_widget(Header, main[0], state);

    // MAIN CONTENT

    // Render border around main content
    Block::default().borders(Borders::ALL)
        .render(main[0], frame.buffer_mut());

    match state.window.main {
        MainWindow::Cover => {
            // Center content horizontally
            let content_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Fill(1), Constraint::Fill(50), Constraint::Fill(1)])
                .split(main[0]);

            // Center content vertically, with room for album cover and album name
            let npl = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(15),
                    Constraint::Min(1),
                ])
                .split(content_layout[1]);

            let now_playing_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    // Cover
                    Constraint::Length(14),
                    // Album name
                    Constraint::Length(1),
                ])
                .split(npl[1]);

            // 35x14
            let cover_rect = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), Constraint::Length(35), Constraint::Min(1)])
                .split(now_playing_layout[0])[1];
            frame.render_stateful_widget(Cover, cover_rect, state);

            frame.render_widget(
                Paragraph::new(state.playback.context_name())
                    .style(Style::default().bold())
                    .alignment(Alignment::Center),
                now_playing_layout[1],
            );
        },
        _ => {}
    }

    if let WindowState::Modal = state.window_state {
        match state.window.modal {
            ModalWindow::DeviceSelect => {
                frame.render_stateful_widget(DeviceSelect, main[0], state);
            },
            _ => {}
        }
    }

    // FOOTER
    frame.render_stateful_widget(Footer, main[1], state);
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

