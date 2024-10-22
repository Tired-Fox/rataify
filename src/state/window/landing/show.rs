use ratatui::widgets::Widget;
use ratatui_image::protocol::StatefulProtocol;
use rspotify::model::Page;

use crate::state::model::Episode;

#[derive(Clone)]
pub struct ShowDetails {
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub name: String,
    pub episodes: Page<Episode>,
}

impl std::fmt::Debug for ShowDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistDetails")
            .field("name", &self.name)
            .field("episodes", &self.episodes)
            .finish_non_exhaustive()
    }
}

impl Widget for &ShowDetails {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        
    }
}
