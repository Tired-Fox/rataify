use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::style::{Style, Stylize};
use crate::state::State;

pub struct Header;

impl StatefulWidget for Header {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        buf.set_string(header[2].left(), header[2].top(), "?", Style::default().bold().italic());
    }
}