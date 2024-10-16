use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::state::actions::GoTo;

use super::{render_modal, KeyToString};

pub struct UiGoto<'a>(pub &'a Vec<(KeyEvent, GoTo)>);

impl<'a> Widget for UiGoto<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized
    {
        render_modal(area, buf, "[Go To]", self.0.iter().map(|(key, goto)| {
            [key.key_to_string(), format!("{:?}", goto)]
        }));
    }
}
