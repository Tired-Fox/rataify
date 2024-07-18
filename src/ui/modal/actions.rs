use ratatui::widgets::Widget;

use super::{render_modal, KeyToString};
use crate::state::modal::ActionState;

impl Widget for &ActionState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        render_modal(area, buf, "[Actions]", self.mappings.iter().map(|(key, _, label)| {
            [key.key_to_string(), label.to_string()]
        }))
    }
}
