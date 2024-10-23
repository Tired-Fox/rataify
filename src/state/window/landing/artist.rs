use ratatui::{layout::{Constraint, Layout}, style::{Style, Stylize}, text::Span, widgets::{Block, Cell, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget}};
use ratatui_image::{protocol::StatefulProtocol, Resize, StatefulImage};
use rspotify::model::Page;

use crate::state::{model::{Album, Artist, Track}, window::{PageRow, Paginatable}};

#[derive(Clone)]
pub struct ArtistDetails {
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub artist: Artist,
    pub albums: Page<Album>,
    pub top_tracks: Vec<Track>,
    pub index: usize,
}


impl std::fmt::Debug for ArtistDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistDetails")
            .field("artist", &self.artist)
            .field("top_tracks", &self.top_tracks)
            .field("albums", &self.albums)
            .finish_non_exhaustive()
    }
}

impl Widget for &mut ArtistDetails {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Span::from(format!("{}x{}", area.width, area.height)).render(area, buf);
        let hoz = Layout::horizontal([Constraint::Length(25), Constraint::Length(1), Constraint::Fill(1)]).split(area);
        let info_layout = Layout::vertical([Constraint::Length(12), Constraint::Length(1), Constraint::Fill(1)]).split(hoz[0]);

        if let Some(image) = self.image.as_mut() {
            let img = StatefulImage::new(None).resize(Resize::Fit(None));
            StatefulWidget::render(img, info_layout[0], buf, image);
        }

        Paragraph::new(self.artist.name.as_str())
            .bold()
            .render(info_layout[1], buf);

        let block = Block::default()
            .padding(Padding::left(2));
        let list_layout = Layout::vertical([Constraint::Length(if area.height < 25 { 5 } else { 10 }), Constraint::Length(1), Constraint::Fill(1)]).split(block.inner(hoz[2]));

        let mut widths: Vec<usize> = Vec::new();
        let lines = self.top_tracks.iter().map(|t| {
            let cells = t.page_row();
            Row::new(cells.into_iter().enumerate().map(|(i, (cell, style))| {
                match widths.get(i).copied() {
                    Some(len) => if cell.len() > len {
                        if let Some(len) = widths.get_mut(i) {
                            *len = cell.len();
                        }
                    },
                    None => widths.push(cell.len())
                }

                match style {
                    Some(s) => s(cell),
                    None => Cell::from(cell)
                }
            }).collect::<Vec<_>>())
        }).collect::<Vec<_>>();

        let mut state = TableState::default().with_selected(if self.index > 9 { None } else { Some(self.index) });
        let top_tracks = Table::new(lines, Track::page_widths(widths))
            .highlight_style(Style::default().yellow());
        StatefulWidget::render(top_tracks, list_layout[0], buf, &mut state);

        let mut state = TableState::default().with_selected(if self.index > 9 { Some(self.index.saturating_sub(10)) } else { None });
        self.albums.paginated(None, 0)
            .render(list_layout[2], buf, &mut state);
    }
}
