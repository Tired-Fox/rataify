use ratatui::widgets::Widget;

use super::render_modal_with_state;
use crate::state::modal::ArtistsState;

impl Widget for &mut ArtistsState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        render_modal_with_state(area, buf, "[Artists]", self.artists.iter().map(|a| {
            [a.1.clone()]
        }), &mut self.state)
    }
}
