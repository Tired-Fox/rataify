use lazy_static::lazy_static;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Tabs, Widget};
use ratatui::Frame;

use crate::state::{MainWindow, ModalWindow, State, WindowState, TABS};
use crate::ui::footer::Footer;
use crate::ui::modal::DeviceSelect;

use self::tabs::{Main, Queue};

mod footer;
pub mod icon;
mod modal;
mod tabs;

pub fn player_ui(state: &mut State, frame: &mut Frame) {
    macro_rules! render_state {
        ($widget: ident, $area: expr) => {
            frame.render_stateful_widget($widget, $area, state)
        };
    }

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(5),
        ])
        .split(frame.size());

    // HEADER
    // frame.render_stateful_widget(Header, main[0], state);
    let current_tab = TABS.iter().position(|t| t == &state.window.main);
    let tabs = Tabs::new(
        TABS.iter()
            .map(|t| format!("{:?}", t))
            .collect::<Vec<String>>(),
    )
    .highlight_style(if current_tab.is_some() {
        Style::default().reversed()
    } else {
        Style::default()
    })
    .select(current_tab.unwrap_or(0));
    frame.render_widget(tabs, main[0]);

    // MAIN CONTENT

    // Render border around main content
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(main[1]);
    block.render(main[1], frame.buffer_mut());

    match state.window.main {
        MainWindow::Cover => render_state!(Main, inner),
        MainWindow::Queue => render_state!(Queue, inner),
        _ => {}
    }

    if let WindowState::Modal = state.window_state {
        match state.window.modal {
            ModalWindow::DeviceSelect => render_state!(DeviceSelect, inner),
            _ => {}
        }
    }

    // FOOTER
    render_state!(Footer, main[2]);
}
