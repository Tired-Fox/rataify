use ratatui::{layout::{Constraint, Layout}, style::Stylize, widgets::{Block, Padding, Paragraph, StatefulWidget, TableState, Widget}};
use ratatui_image::{protocol::StatefulProtocol, Resize, StatefulImage};
use rspotify::model::Page;

use crate::state::{model::{Item, Playlist}, window::Paginatable};

#[derive(Clone)]
pub struct PlaylistDetails {
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub playlist: Playlist,
    pub items: Page<Item>,
    pub index: usize,
}
impl std::fmt::Debug for PlaylistDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistDetails")
            .field("playlist", &self.playlist)
            .field("items", &self.items)
            .finish_non_exhaustive()
    }
}

impl Widget for &mut PlaylistDetails {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let hoz = Layout::horizontal([Constraint::Length(25), Constraint::Length(1), Constraint::Fill(1)]).split(area);
        let info_layout = Layout::vertical([Constraint::Length(12), Constraint::Length(1), Constraint::Fill(1)]).split(hoz[0]);

        if let Some(image) = self.image.as_mut() {
            let img = StatefulImage::new(None).resize(Resize::Fit(None));
            StatefulWidget::render(img, info_layout[0], buf, image);
        }

        Paragraph::new(self.playlist.name.as_str())
            .bold()
            .render(info_layout[1], buf);

        let mut state = TableState::default().with_selected(Some(self.index));
        let block = Block::default()
            .padding(Padding::left(2));
        self.items.paginated(None, self.index)
            .render(block.inner(hoz[2]), buf, &mut state);
    }
}
