use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Layout, Rect, Margin}, symbols::border, widgets::{block::Title, Block, Cell, Clear, Padding, Row, StatefulWidget, Table, TableState, Widget, Scrollbar, ScrollbarState, ScrollbarOrientation}};

use tupy::api::response::Paged;

use crate::state::{modal::AddToPlaylistState, Loading};
use crate::ui::PaginationProgress;

use super::COLORS;

impl Widget for &mut AddToPlaylistState {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
        let title = "[Add To Playlist]";
        let rows = if let Some(Loading::Some(playlists)) = self.playlists.items.lock().unwrap().as_ref().map(|p| p.as_ref()) {
            playlists.items.iter().map(|i| {
                [i.name.clone()]
            }).collect::<Vec<_>>()
        } else { vec![] };

        let len = rows.len();

        let mut longest_parts: [usize; 1] = [0; 1];

        // Rernder in bottom right corner
        // [{key}] {title}
        let mut count = 0;
        let list = rows.into_iter().map(|parts| {
            count += 1;
            let cells = parts.into_iter().enumerate().map(|(i, part)| {
                if part.len() > longest_parts[i] {
                    longest_parts[i] = part.len();
                }

                Cell::from(part)
            });

            Row::new(cells)
        })
            .collect::<Table>()
            .block(Block::bordered()
                //.borders(Borders::TOP | Borders::LEFT)
                .border_set(border::ROUNDED)
                .padding(Padding::symmetric(1, 1))
                .title(Title::from(title).alignment(Alignment::Center))
            )
            .highlight_style(COLORS.highlight)
            .widths(longest_parts.iter().map(|l| Constraint::Length(*l as u16)))
            .column_spacing(2);

        let hoz = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length((longest_parts.iter().sum::<usize>() as u16 + 6).max(title.len() as u16 + 2)),
            Constraint::Length(1),
        ])
            .split(area);

        let height = 14.min(area.height - 4);
        let vert = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Length(1),
        ])
            .split(hoz[1]);

        Clear.render(vert[1], buf);
        StatefulWidget::render(list, vert[1], buf, &mut self.state);

        if let Some(Loading::Some(playlists)) = self.playlists.items.lock().unwrap().as_ref().map(|p| p.as_ref()) {
            PaginationProgress {
                current: playlists.page(),
                total: playlists.max_page(),
            }
                .render(vert[1], buf);
        }

        if len >= height as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scroll_state = ScrollbarState::new(len).position(self.state.selected().unwrap_or(0));
            StatefulWidget::render(
                scrollbar,
                vert[1].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                buf,
                &mut scroll_state,
            );
        }
    }
}
