use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap
    },
};
use tupy::api::response::{Artist, ArtistAlbums, Paged, SimplifiedAlbum, Track};

use crate::{
    state::{
        window::{
            landing::{ArtistLanding, Cover, LandingSection},
            MappedPages,
        }, wrappers::Saved, Loading
    },
    ui::{format_track_saved, PaginationProgress, COLORS},
    Locked, Shared,
};

use super::{render_landing, COMPACT};

#[allow(clippy::too_many_arguments)]
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    artist: &Saved<Artist>,
    top_tracks: &[Saved<Track>],
    albums: &MappedPages<Vec<Saved<SimplifiedAlbum>>, ArtistAlbums, ArtistAlbums>,
    state: &TableState,
    section: &ArtistLanding,
    landing_section: &LandingSection,
    cover: &Shared<Locked<Loading<Cover>>>,
) {
    let title = format!("[Artist: {}]", artist.as_ref().name);
    let (under, main) = render_landing(
        area,
        buf,
        title.clone(),
        cover.clone(),
    );
    
    let followers = artist.as_ref().followers.total.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");

    let info_highlight = if let LandingSection::Context = landing_section { COLORS.highlight } else { Style::default() };

    let info = [
        Paragraph::new(Line::from(if artist.saved { "♥" } else { "" }).centered().style(COLORS.like)),
        if under.height <= 4 {
            Paragraph::new(artist.as_ref().genres.join(", ")).style(info_highlight)
        } else {
            Paragraph::new(artist.as_ref().genres.join(", ")).style(info_highlight).wrap(Wrap { trim: true })
        },
        Paragraph::new(format!("{followers} Followers")).style(info_highlight),
        Paragraph::new(format!("Popularity {}%", artist.as_ref().popularity)).style(info_highlight),
    ];

    if under.height <= info.len() as u16 {
        let info_vert = Layout::vertical(vec![Constraint::Length(1); under.height as usize])
            .split(under);

        for i in 0..under.height as usize {
            (&info[i]).render(info_vert[i], buf);
        }
    } else {
        let mut constraints = vec![Constraint::Length(1), Constraint::Fill(1)];
        constraints.extend(vec![Constraint::Length(1); info.len() - 2]);
        let info_vert = Layout::vertical(constraints).split(under);

        (&info[0]).render(info_vert[0], buf);
        for i in 1..info.len() {
            (&info[i]).render(info_vert[i], buf);
        }
    };

    let vert = Layout::vertical([
        if main.height <= COMPACT {
            Constraint::Length(5)
        } else {
            Constraint::Length(10)
        },
        Constraint::Fill(1),
    ])
    .split(main);

    // RENDER ARTIST'S TOP TRACKS
    let table_top_tracks = top_tracks
        .iter()
        .map(|v| format_track_saved(v.as_ref(), v.saved))
        .collect::<Table>()
        .widths([
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ])
        .highlight_style(if landing_section.is_content() && section.is_tracks() { COLORS.highlight } else { Style::default() });

        match section {
            ArtistLanding::Tracks => {
                StatefulWidget::render(table_top_tracks, vert[0], buf, &mut state.clone());
            }
            ArtistLanding::Albums => {
                let mut temp_state = TableState::default();
                temp_state.select(Some(top_tracks.len() - 1));
                StatefulWidget::render(table_top_tracks, vert[0], buf, &mut temp_state);
            }
        }
    

    // RENDER ALBUMS
    match albums.items.lock().unwrap().as_ref() {
        Some(Loading::Loading) => {
            let vert = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(vert[1])[1];

            Line::from("Loading...").centered().render(vert, buf);
        }
        None | Some(Loading::None) => {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                .split(vert[1])[1];

            Line::from("<No Playlist Items>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => {
            let page = albums.page.lock().unwrap();
            let scrollable = page.limit >= vert[1].height as usize;
            let block = Block::default().padding(Padding::new(0, if scrollable { 2 } else { 0 }, 0, 1));

            let table_albums = data
                .iter()
                .map(|a| {
                    Row::new(vec![
                        Cell::from(if a.saved { "♥" } else { "" }).style(COLORS.like),
                        Cell::from(a.as_ref().name.clone()).style(COLORS.context),
                        Cell::from(format!("{:?}", a.as_ref().album_type)),
                        Cell::from(
                            a.as_ref().artists
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
                    Constraint::Length(2),
                    Constraint::Fill(3),
                    Constraint::Length(11),
                    Constraint::Fill(1),
                ])
                .highlight_style(if landing_section.is_content() { COLORS.highlight } else { Style::default() });

            PaginationProgress {
                current: page.page,
                total: page.max_page,
            }
            .render(vert[1], buf);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = match section {
                ArtistLanding::Tracks => {
                    Widget::render(table_albums, vert[1], buf);
                    ScrollbarState::new(data.len()).position(0)
                }
                ArtistLanding::Albums => {
                    StatefulWidget::render(table_albums, vert[1], buf, &mut state.clone());
                    ScrollbarState::new(data.len()).position(state.selected().unwrap_or(0))
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
