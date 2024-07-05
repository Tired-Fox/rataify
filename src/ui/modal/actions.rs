use ratatui::widgets::Widget;

use crate::ui::action::Action;
use crossterm::event::KeyCode;

use super::render_modal;

trait KeyToString {
    fn key_to_string(&self) -> String;
}

impl KeyToString for KeyCode {
    fn key_to_string(&self) -> String {
        match self {
            KeyCode::Char(c) => c.to_string(),
            other => format!("{:?}", other),
        }
    }
}

pub struct ModalActions<'a>(pub &'a Vec<Action>);
impl<'a> Widget for ModalActions<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        render_modal(area, buf, "[Actions]", self.0.iter().map(|a| {
            [a.with_key().key_to_string(), a.to_string()]
        }))
    }
}
