use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::Stylize,
    symbols::border::{self, Set},
    text::{Line, Span},
    widgets::{
        Wrap, Paragraph,
        block::{Position, Title},
        Block, Cell, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Table, TableState, Widget,
    },
};
use ratatui_image::Image;
use tupy::api::response::{Artist, ArtistAlbums, Paged, Track};

use crate::{
    state::{
        window::{
            landing::{ArtistLanding, Cover},
            Pages,
        },
        Loading,
    },
    ui::{format_track, PaginationProgress, COLORS},
    Locked, Shared,
};

static DASHED: Set = Set {
    vertical_left: "┆",
    vertical_right: "┆",
    horizontal_top: "┄",
    horizontal_bottom: "┄",
    ..border::ROUNDED
};

static INFO_CUTOFF: u16 = 18;
static COMPACT: u16 = 21;

#[allow(clippy::too_many_arguments)]
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    artist: &Artist,
    top_tracks: &[Track],
    albums: &Pages<ArtistAlbums, ArtistAlbums>,
    state: &TableState,
    section: &ArtistLanding,
    cover: &Shared<Locked<Loading<Cover>>>,
) {
    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .padding(Padding::symmetric(1, 1))
        .title(
            Title::from(format!("[Artist: {}]", artist.name))
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

    let info = [
        if info_area.height <= 3 {
            Paragraph::new(artist.genres.join(", "))
        } else {
            Paragraph::new(artist.genres.join(", ")).wrap(Wrap { trim: true })
        },
        Paragraph::new(format!("{} Followers", artist.followers.total)),
        Paragraph::new(format!("Popularity {}%", artist.popularity)),
    ];

    if info_area.height <= info.len() as u16 {
        let info_vert = Layout::vertical(vec![Constraint::Length(1); info_area.height as usize])
            .split(info_area);

        for i in 0..info_area.height as usize {
            (&info[i]).render(info_vert[i], buf);
        }
    } else {
        let mut constraints = vec![Constraint::Fill(1)];
        constraints.extend(vec![Constraint::Length(1); info.len() - 1]);
        let info_vert = Layout::vertical(constraints).split(info_area);

        (&info[0]).render(info_vert[0], buf);
        for i in 1..info.len() {
            (&info[i]).render(info_vert[i], buf);
        }
    };

    let vert = Layout::vertical([
        if area.height <= COMPACT {
            Constraint::Length(5)
        } else {
            Constraint::Length(10)
        },
        Constraint::Fill(1),
    ])
    .split(hoz[2]);

    // RENDER ARTIST'S TOP TRACKS
    let table_top_tracks = top_tracks
        .iter()
        .map(|t| format_track(t))
        .collect::<Table>()
        .widths([
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ])
        .highlight_style(COLORS.highlight);

    match section {
        ArtistLanding::Tracks => {
            StatefulWidget::render(table_top_tracks, vert[0], buf, &mut state.clone());
        }
        ArtistLanding::Albums => {
            Widget::render(table_top_tracks, vert[0], buf);
        }
    }

    // RENDER ALBUMS
    match albums.items.lock().unwrap().as_ref() {
        Some(Loading::Loading) => {
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .padding(Padding::symmetric(1, 1))
                .title(
                    Title::from("[Artist]")
                        .alignment(Alignment::Center)
                        .position(Position::Bottom),
                );
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
                .title(
                    Title::from("[Artist]")
                        .alignment(Alignment::Center)
                        .position(Position::Bottom),
                );
            (&block).render(area, buf);
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)])
                .split(block.inner(area))[1];

            Line::from("<No Playlist Items>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => {
            let scrollable = data.limit() >= vert[1].height as usize;
            let block = Block::default().padding(Padding::new(0, if scrollable { 2 } else { 0 }, 1, 0));

            let table_albums = data
                .items
                .iter()
                .map(|a| {
                    Row::new(vec![
                        Cell::from(a.name.clone()).style(COLORS.context),
                        Cell::from(format!("{:?}", a.album_type)),
                        Cell::from(
                            a.artists
                                .iter()
                                .map(|a| a.name.clone())
                                .collect::<Vec<_>>()
                                .join(", "),
                        )
                        .style(COLORS.artists),
                    ])
                })
                .collect::<Table>()
                .block(block)
                .widths([
                    Constraint::Fill(3),
                    Constraint::Length(11),
                    Constraint::Fill(1),
                ])
                .highlight_style(COLORS.highlight);

            PaginationProgress {
                current: data.page(),
                total: data.max_page(),
            }
            .render(vert[1], buf);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = match section {
                ArtistLanding::Tracks => {
                    Widget::render(table_albums, vert[1], buf);
                    ScrollbarState::new(data.items.len()).position(0)
                }
                ArtistLanding::Albums => {
                    StatefulWidget::render(table_albums, vert[1], buf, &mut state.clone());
                    ScrollbarState::new(data.items.len()).position(state.selected().unwrap_or(0))
                }
            };

            if scrollable {
                StatefulWidget::render(
                    scrollbar,
                    vert[1].inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    buf,
                    &mut scrollbar_state,
                );
            }
        }
    }
}
