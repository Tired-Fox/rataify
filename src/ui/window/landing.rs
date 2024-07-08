use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect, Margin},
    style::{Color, Style, Stylize, Styled},
    symbols::border,
    text::{Line, Span},
    widgets::{
        block::{Block, Padding, Position, Title}, Cell, Row, StatefulWidget, Table, TableState, Widget,
        Scrollbar, ScrollbarState, ScrollbarOrientation
    },
};
use tupy::api::response::{Item, PlaylistItems, AlbumTracks, Paged, ShowEpisodes, Chapters};

use crate::{
    ui::{format_episode, format_track, format_duration, COLORS, PaginationProgress},
    state::{window::{landing::Landing, Pages}, Loading}
};

struct LandingData<'a> {
    progress: PaginationProgress,
    table: Table<'a>,
    total: usize,
    state: TableState,
    title: &'static str,
}

impl Widget for LandingData<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .padding(Padding::symmetric(1, 1))
            .title(Title::from(format!("[{}]", self.title)).alignment(Alignment::Center).position(Position::Bottom));

        let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)]).split(area)[1];
        self.progress.render(vert, buf);

        let scrollable = self.total > block.inner(area).height as usize;
        let table = self.table
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow))
            .column_spacing(2);

        StatefulWidget::render(table, area, buf, &mut self.state);

        if scrollable {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(self.total).position(self.state.selected().unwrap_or(0));
            StatefulWidget::render(scrollbar, area.inner(Margin { vertical: 1, horizontal: 0 }), buf, &mut scrollbar_state);
        }
    }
}

impl Widget for &mut Landing {
fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Landing::None => {},
            Landing::Playlist(pages, state) => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, "Playlist", area,buf)
            },
            Landing::Album(pages, state) => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, "Album", area,buf)
            },
            Landing::Show(pages, state) => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, "Show", area,buf)
            },
            Landing::Audiobook(pages, state) => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, "Audiobook", area,buf)
            },
            // TODO: This one will probably have a mix of albums, tracks, etc.
            Landing::Artist => {
                todo!("Artist requires more work since it has a mix of albums, tracks, etc.")
            }
        }
    }
}

trait IntoTable<'a> {
    fn into_table(self) -> Table<'a>;
}

impl<'a> IntoTable<'a> for &'a PlaylistItems {
    fn into_table(self) -> Table<'a> {
        self.items
            .iter()
            .map(|item| {
                match &item.item {
                    Item::Track(track) => format_track(&track),
                    Item::Episode(episode) => format_episode(&episode),
                }
            })
            .collect::<Table>()
            .widths([
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Fill(2),
            ])
    }
}

impl<'a> IntoTable<'a> for &'a AlbumTracks {
    fn into_table(self) -> Table<'a> {
        self.items
            .iter()
            .map(|track| {
                Row::new(vec![
                    Cell::from(format_duration(track.duration)).style(COLORS.duration),
                    Cell::from(track.name.clone()).style(COLORS.track),
                    Cell::from(track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")).style(COLORS.artists),
                ])
            })
            .collect::<Table>()
            .widths([
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Fill(2),
            ])
    }
}

impl<'a> IntoTable<'a> for &'a ShowEpisodes {
    fn into_table(self) -> Table<'a> {
        self.items
            .iter()
            .map(|e| {
                let mut cells = vec![
                    Cell::from(format_duration(e.duration)).style(COLORS.duration),
                    if e.resume_point.fully_played {
                        Cell::from(" ✓").style(COLORS.finished)
                    } else {
                        Cell::default()
                    },
                    Cell::from(e.name.clone()).style(COLORS.episode),
                ];

                Row::new(cells)
            })
            .collect::<Table>()
            .widths([
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Fill(2),
            ])
    }
}

impl<'a> IntoTable<'a> for &'a Chapters {
    fn into_table(self) -> Table<'a> {
        self.items
            .iter()
            .map(|e| {
                let mut cells = vec![
                    // duration, chapter, name, finished
                    Cell::from(format_duration(e.duration)).style(COLORS.duration),
                    Cell::from(format!("Chapter {}", e.chapter_number)).style(COLORS.chapter_number),
                    Cell::from(e.name.clone()).style(COLORS.episode),
                    if e.resume_point.fully_played {
                        Cell::from(Line::from(vec![
                            Span::from(e.name.clone()).style(COLORS.episode),
                            Span::from(" ✓").style(COLORS.finished)
                        ]))
                    } else {
                        Cell::from(e.name.clone()).style(COLORS.episode)
                    }
                ];

                Row::new(cells)
            })
            .collect::<Table>()
            .widths([
                Constraint::Length(8),
                Constraint::Length(8 + format!("{}", self.total).len() as u16),
                Constraint::Fill(1),
                Constraint::Fill(2),
            ])
    }
}

fn unwrap_or_render<'a, T: IntoTable<'a> + Paged>(data: Option<Loading<T>>, state: &mut TableState, title: &'static str, area: Rect, buf: &mut Buffer) {
    match data {
        Some(Loading::Loading) => {
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .padding(Padding::symmetric(1, 1))
                .title(Title::from(format!("[{}]", title)).alignment(Alignment::Center).position(Position::Bottom));
            (&block).render(area, buf);
            let vert = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
                .split(block.inner(area))[1];

            Line::from("Loading...").centered().render(vert, buf);
        }
        None | Some(Loading::None) => {
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .padding(Padding::symmetric(1, 1))
                .title(Title::from(format!("[{}]", title)).alignment(Alignment::Center).position(Position::Bottom));
            (&block).render(area, buf);
            let vert = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
                .split(block.inner(area))[1];

            Line::from("<No Playlist Items>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => LandingData {
            progress: PaginationProgress {
                current: data.page(),
                total: data.max_page(),
            },
            total: data.items().len(),
            state: state.clone(),
            table: data.into_table(),
            title: title,
        }.render(area, buf),
    }
}