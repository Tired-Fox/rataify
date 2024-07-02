use ratatui::widgets::Widget;

use crate::state::DevicesState;

use super::render_modal_with_state;

impl Widget for &mut DevicesState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        render_modal_with_state(area, buf, "[Devices]", self.devices.iter().map(|d| {
            [d.name.clone(), format!("[{}]", d.device_type)]            
        }), &mut self.state);
    }
}
