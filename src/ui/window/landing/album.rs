use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap
    },
};
use tupy::api::response::{Album, AlbumTracks, ReleaseDate, SimplifiedTrack};

use crate::{
    state::{
        window::{
            landing::{Cover, LandingSection}, MappedPages
        }, wrappers::Saved, Loading
    },
    ui::{format_duration, PaginationProgress, COLORS},
    Locked, Shared,
};

use super::render_landing;

#[allow(clippy::too_many_arguments)]
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    album: &Album,
    pages: &MappedPages<Vec<Saved<SimplifiedTrack>>, AlbumTracks, AlbumTracks>,
    state: &TableState,
    section: &LandingSection,
    cover: &mut Shared<Locked<Loading<Cover>>>,
) {
    let title = format!("[Album: {}]", album.name);
    let (under, main) = render_landing(
        area,
        buf,
        title.clone(),
        cover.clone(),
    );

    let artists = album.artists.iter().map(|v| v.name.clone()).collect::<Vec<_>>().join(", ");
    let info_highlight = if let LandingSection::Context = section { COLORS.highlight } else { Style::default() };
    let info = [
        if under.height <= 2 {
            Paragraph::new(artists).style(info_highlight)
        } else {
            Paragraph::new(artists).style(info_highlight).wrap(Wrap { trim: true })
        },
        Paragraph::new(format!("Released: {}", match album.release {
            ReleaseDate::Day(date) => date.format("%b %d, %Y"),
            ReleaseDate::Month(date) => date.format("%b, %Y"),
            ReleaseDate::Year(date) => date.format("%Y"),
        })).style(info_highlight).bold(),
    ];
    
    if under.height <= info.len() as u16 {
        let info_vert = Layout::vertical(vec![Constraint::Length(1); under.height as usize])
            .split(under);

        for i in 0..under.height as usize {
            (&info[i]).render(info_vert[i], buf);
        }
    } else {
        let mut constraints = vec![Constraint::Fill(1)];
        constraints.extend(vec![Constraint::Length(1); info.len() - 1]);
        let info_vert = Layout::vertical(constraints).split(under);

        (&info[0]).render(info_vert[0], buf);
        for i in 1..info.len() {
            (&info[i]).render(info_vert[i], buf);
        }
    };
    
    // RENDER ALBUM TRACKS
    //
    match pages.items.lock().unwrap().as_ref() {
        Some(Loading::Loading) => {
            let vert = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(main)[1];

            Line::from("Loading...").centered().render(vert, buf);
        }
        None | Some(Loading::None) => {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                .split(main)[1];

            Line::from("<No Album Items>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => {
            let page = pages.page.lock().unwrap();
            let scrollable = page.limit >= main.height as usize;
            let block = Block::default().padding(Padding::new(0, if scrollable { 2 } else { 0 }, 0, 1));

            let table_tracks = data
                .iter()
                .map(|track| {
                    Row::new(vec![
                        Cell::from(if track.saved { "♥" } else { "" }).style(COLORS.like),
                        Cell::from(format_duration(track.as_ref().duration)).style(COLORS.duration),
                        Cell::from(track.as_ref().name.clone()).style(COLORS.track),
                        Cell::from(track.as_ref().artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")).style(COLORS.artists),
                    ])
                })
                .collect::<Table>()
                .widths([
                    Constraint::Length(2),
                    Constraint::Length(8),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ])
                .block(block)
                .highlight_style(if section.is_content() { COLORS.highlight } else { Style::default() });

            StatefulWidget::render(table_tracks, main, buf, &mut state.clone());

            PaginationProgress {
                current: page.page,
                total: page.max_page,
            }
            .render(main, buf);

            if scrollable {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(data.len()).position(state.selected().unwrap_or(0));
                StatefulWidget::render(
                    scrollbar,
                    main.inner(Margin {
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
