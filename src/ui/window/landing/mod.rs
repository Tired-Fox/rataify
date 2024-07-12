use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols::{border, line},
    text::{Line, Span},
    widgets::{
        block::{Block, Padding, Position, Title}, Cell, Clear, LineGauge, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget 
    },
};
use ratatui_image::Image;
use tupy::api::response::{Item, PlaylistItems, AlbumTracks, Paged, ShowEpisodes, Chapters};

use crate::{
    state::{window::{landing::{ArtistLanding, Cover, Landing}, Pages}, Loading}, ui::{format_duration, format_episode, format_track, PaginationProgress, COLORS}, Locked, Shared
};

mod artist;
mod playlist;

struct LandingData<'a> {
    progress: PaginationProgress,
    table: Table<'a>,
    total: usize,
    state: TableState,
    title: String,
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
            .highlight_style(COLORS.highlight)
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
            Landing::Playlist{ pages, state, playlist, cover } => {
                playlist::render(area, buf, playlist, pages, state, cover);
            },
            Landing::Album{ pages, state, album, .. } => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, format!("Album: {}", album.name), area,buf)
            },
            Landing::Show{ pages, state, show, .. } => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, format!("Show: {}", show.name), area,buf)
            },
            Landing::Audiobook{ pages, state, audiobook, .. } => {
                let result = pages.items.lock().unwrap();
                unwrap_or_render(result.as_ref().map(|p| p.as_ref()), state, format!("Audiobook: {}", audiobook.name), area,buf)
            },
            // TODO: This one will probably have a mix of albums, tracks, etc.
            Landing::Artist { top_tracks, state, section, albums, artist, cover } => {
                artist::render(area, buf, artist, top_tracks, albums, state, section, cover);
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
                Constraint::Length(1),
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
                Row::new(vec![
                    Cell::from(format_duration(e.duration)).style(COLORS.duration),
                    if e.resume_point.fully_played {
                        Cell::from("✓").style(COLORS.finished)
                    } else {
                        Cell::default()
                    },
                    Cell::from(e.name.clone()).style(COLORS.episode),
                ])
            })
            .collect::<Table>()
            .widths([
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Fill(2),
            ])
    }
}

impl<'a> IntoTable<'a> for &'a Chapters {
    fn into_table(self) -> Table<'a> {
        self.items
            .iter()
            .map(|e| {
                Row::new(vec![
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
                ])
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

fn unwrap_or_render<'a, T: IntoTable<'a> + Paged>(data: Option<Loading<T>>, state: &mut TableState, title: String, area: Rect, buf: &mut Buffer) {
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

static INFO_CUTOFF: u16 = 18;
static COMPACT: u16 = 21;

fn render_landing(
    area: Rect,
    buf: &mut Buffer,
    title: String,
    cover: Shared<Locked<Loading<Cover>>>,
) -> (Rect, Rect)
{
    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .padding(Padding::symmetric(1, 1))
        .title(
            Title::from(title)
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        );

    (&block).render(area, buf);
    let inner = block.inner(area);

    let hoz = Layout::horizontal([
        Constraint::Length(28),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(inner);

    // RENDER ARTIST INFORMATION
    let info_area = match cover.lock().unwrap().as_ref() {
        Loading::Some(cover) => {
            let lyt = Layout::vertical([
                Constraint::Length(14),
                if area.height < INFO_CUTOFF {
                    Constraint::Length(0)
                } else {
                    Constraint::Fill(1)
                },
            ])
            .split(hoz[0]);
            Image::new(cover.image.as_ref()).render(lyt[0], buf);
            lyt[1]
        }
        Loading::Loading => Layout::vertical([
            Constraint::Length(14),
            if area.height < INFO_CUTOFF {
                Constraint::Length(0)
            } else {
                Constraint::Fill(1)
            },
        ])
        .split(hoz[0])[1],
        Loading::None => hoz[0],
    };

    (info_area, hoz[2])
}
