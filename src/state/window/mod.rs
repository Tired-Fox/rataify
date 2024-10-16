pub mod library;
pub mod modal;

use ratatui::{
    style::{Style, Stylize},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use super::InnerState;

#[derive(Default, Debug, Clone, Copy)]
pub enum Window {
    #[default]
    Library,
    Modal(modal::Modal),
}

impl StatefulWidget for Window {
    type State = InnerState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().dark_gray());

        Widget::render(&block, area, buf);

        match state.window {
            Window::Library => library::Library.render(block.inner(area), buf, state),
            Window::Modal(modal) => modal.render(area, buf, state)
        }
    }
}
