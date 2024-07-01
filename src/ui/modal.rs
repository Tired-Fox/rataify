use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style, Stylize}, symbols::border, widgets::{block::{Position, Title}, Block, Clear, List, ListDirection, Padding, StatefulWidget, Widget}
};

use crate::state::DevicesState;

impl Widget for &mut DevicesState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {

        let devices = self.devices
            .iter()
            .map(|d| format!("{} [{}]", d.name, d.device_type))
            .collect::<Vec<_>>();

        let widest = devices.iter().max().unwrap().len() as u16;
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::vertical([
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Percentage(50),
            Constraint::Percentage((100 - 50) / 2),
        ])
            .split(area);

        // Then cut the middle vertical piece into three width-wise pieces
        let modal = Layout::horizontal([
            Constraint::Fill(1),
            if area.width as f32 * 0.25 < (widest + 4) as f32 { Constraint::Length(widest + 4) } else { Constraint::Percentage(25) },
            Constraint::Fill(1),
        ])
            .split(popup_layout[1])[1];

        // TODO: Handle rendering in scrollable area ???
        let list = self
            .devices
            .iter()
            .map(|d| format!("{} [{}]", d.name, d.device_type))
            .collect::<List>()
            .block(
                Block::bordered()
                    .title(
                        Title::from("[Devices]".bold())
                            .alignment(Alignment::Center)
                            .position(Position::Bottom)
                    )
                    .padding(Padding::new(1, 1, 0, 0))
                    .border_set(border::ROUNDED),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .direction(ListDirection::TopToBottom);
        
        Clear.render(modal, buf);
        StatefulWidget::render(list, modal, buf, &mut self.state);
    }
}
