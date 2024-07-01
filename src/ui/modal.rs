use ratatui::{style::{Color, Modifier, Style}, symbols::border, widgets::{Block, List, ListDirection, StatefulWidget, Widget}};

use crate::app::Modal;

use super::centered_rect;

impl Widget for &mut Modal {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        match self {
            Modal::Devices{ state, devices } => {
                // TODO: Handle rendering in scrollable area ???
                let list = devices.iter()
                    .map(|d| format!("{} [{}]", d.name, d.device_type))
                    .collect::<List>()
                    .block(Block::bordered().title("Devices").border_set(border::ROUNDED))
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
                    .direction(ListDirection::TopToBottom);

                StatefulWidget::render(list, centered_rect(25, 50, area), buf, state);
            }
        }
    }
}
