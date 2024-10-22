use ratatui::widgets::Widget;
use ratatui_image::protocol::StatefulProtocol;
use rspotify::model::{Page, SimplifiedTrack};

use crate::state::model::Track;

#[derive(Clone)]
pub struct AlbumDetails  {
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub name: String,
    pub tracks: Page<Track>,
}

impl std::fmt::Debug for AlbumDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistDetails")
            .field("name", &self.name)
            .field("tracks", &self.tracks)
            .finish_non_exhaustive()
    }
}

impl Widget for &AlbumDetails {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        
    }
}
