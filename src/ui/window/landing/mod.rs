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
mod album;
mod show;
mod audiobook;

lazy_static::lazy_static! {
    pub static ref HTML_UNICODE: regex::Regex = regex::Regex::new("&#(?:(?<decimal>[0-9]+)|x(?<hex>[0-9a-fA-F]+));").unwrap();
}

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
            Landing::Playlist{ pages, state, playlist, cover, section } => {
                playlist::render(area, buf, playlist, pages, state, section, cover);
            },
            Landing::Album{ pages, state, album, cover, section } => {
                album::render(area, buf, album, pages, state, section, cover);
            },
            Landing::Show{ pages, state, show, cover, section } => {
                show::render(area, buf, show, pages, state, section, cover);
            },
            Landing::Audiobook{ pages, state, audiobook, cover, section } => {
                audiobook::render(area, buf, audiobook, pages, state, section, cover);
            },
            Landing::Artist { top_tracks, state, section, albums, artist, cover, landing_section } => {
                artist::render(area, buf, artist, top_tracks, albums, state, section, landing_section, cover);
            }
        }
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
        Constraint::Length(30),
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
