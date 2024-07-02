use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::ui::action::GoTo;

use super::render_modal;

pub struct UiGoto<'a>(pub &'a Vec<(KeyCode, GoTo)>);

impl<'a> Widget for UiGoto<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized
    {
        render_modal(area, buf, "[Go To]", self.0.iter().map(|(key, goto)| {
            let key = format!("[{}]", match key {
                KeyCode::F(f) => format!("F{}", f),
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Media(_) | KeyCode::Modifier(_) => String::new(),
                other => format!("{:?}", other).to_ascii_lowercase(),
            });
            let title = format!("{:?}", goto);

            [key, title]
        }));
    }
}
