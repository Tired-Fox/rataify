use ratatui::widgets::Widget;

use crate::ui::action::UiAction;

use super::render_modal;

pub struct ModalActions<'a>(pub &'a Vec<UiAction>);

impl<'a> Widget for ModalActions<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        render_modal(area, buf, "[Actions]", self.0.iter().map(|a| {
            [a.with_key().to_string(), a.to_string()]
        }))
    }
}
