use ratatui::widgets::Widget;
use tupy::api::Uri;

use super::render_modal;

pub struct AddToPlaylist<'a>(pub Option<&'a Uri>);

impl<'a> Widget for AddToPlaylist<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        render_modal(area, buf, "[Playlists]", [[]])
    }
}
