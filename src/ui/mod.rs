use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Tabs, Widget};
use ratatui::Frame;
use ratatui::prelude::StatefulWidget;

use crate::state::{MainWindow, ModalWindow, State, WindowState, TABS};
use crate::ui::footer::Footer;
use crate::ui::modal::DeviceSelect;

use self::tabs::{Main, Queue};

mod footer;
pub mod icon;
mod modal;
mod tabs;
mod list_view;

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
            Constraint::Length(6),
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


// TODO: Implement set pattern algorithms
// TODO: Add color randomness potential
pub struct Cover;
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
                    .set_symbol(
                        &cover
                            .get(start_y + y as usize)
                            .unwrap()
                            .get(start_x + x as usize)
                            .unwrap()
                            .to_string(),
                    );
            }
        }

        buf.get_mut(rect.left(), rect.top()).set_symbol("┌─");
        buf.get_mut(rect.right() - 2, rect.top()).set_symbol("─┐");
        buf.get_mut(rect.left(), rect.bottom() - 1).set_symbol("└─");
        buf.get_mut(rect.right() - 2, rect.bottom() - 1)
            .set_symbol("─┘");
    }
}
