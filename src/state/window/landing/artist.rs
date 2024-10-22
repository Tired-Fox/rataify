use ratatui::widgets::Widget;
use ratatui_image::protocol::StatefulProtocol;
use rspotify::model::Page;

use crate::state::model::{Album, Track};

#[derive(Clone)]
pub struct ArtistDetails {
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub name: String,
    pub albums: Page<Album>,
    pub top_tracks: Vec<Track>,
}


impl std::fmt::Debug for ArtistDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistDetails")
            .field("name", &self.name)
            .field("top_tracks", &self.top_tracks)
            .field("albums", &self.albums)
            .finish_non_exhaustive()
    }
}

impl Widget for &ArtistDetails {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        
    }
}
