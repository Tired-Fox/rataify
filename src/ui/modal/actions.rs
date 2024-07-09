use ratatui::widgets::Widget;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventState};

use super::{render_modal, KeyToString};
use crate::state::modal::ActionState;

impl Widget for &ActionState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        render_modal(area, buf, "[Actions]", self.mappings.iter().map(|(key, action)| {
            [key.key_to_string(), action.to_string()]
        }))
    }
}
