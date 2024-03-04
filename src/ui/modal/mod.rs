use crate::state::State;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Clear, StatefulWidget, Widget};
use std::cmp::max;

pub struct DeviceSelect;
impl StatefulWidget for DeviceSelect {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // NOTE: Area is the container where the modal can be rendered, not the area of the modal
        let names = state
            .device_select
            .devices
            .iter()
            .map(|d| d.name.clone())
            .collect::<Vec<_>>();

        let max_width = names
            .iter()
            .map(|n| n.chars().count())
            .max()
            .unwrap_or(20)
            .max(20);

        let block = Block::default()
            .title("Devices")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        let mut modal_vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Percentage(80),
                Constraint::Fill(1),
            ])
            .split(area)[1];

        if block.inner(modal_vert).height as usize > names.len() {
            modal_vert = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length((names.len() as u16 + 2).min(area.height)),
                    Constraint::Fill(1),
                ])
                .split(area)[1];
        }

        let modal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length((max_width as u16 + 2).min(area.width)),
                Constraint::Fill(1),
            ])
            .split(modal_vert)[1];

        // Use this inner area to render the content
        let inner = block.inner(modal);
        block.render(modal, buf);
        Clear.render(inner, buf);

        for y in 0..inner.height {
            let mut line = Span::default().content(&names[y as usize]);
            if y as usize == state.device_select.selection {
                line = line.style(Style::default().reversed());
            }

            buf.set_span(inner.left(), inner.top() + y, &line, line.width() as u16);
        }

        // TODO: Render device list with selection
    }
}
