use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap
    },
};
use tupy::api::response::{Playlist, PlaylistItems, Paged, Item};

use crate::{
    state::{
        window::{
            landing::{Cover, LandingSection},
            Pages,
        },
        Loading,
    },
    ui::{format_episode, format_track, PaginationProgress, COLORS},
    Locked, Shared,
};

use super::{render_landing, HTML_UNICODE};

#[allow(clippy::too_many_arguments)]
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    playlist: &Playlist,
    pages: &Pages<PlaylistItems, PlaylistItems>,
    state: &TableState,
    section: &LandingSection,
    cover: &mut Shared<Locked<Loading<Cover>>>,
) {
    let title = format!("[Playlist: {}]", playlist.name);
    let (under, main) = render_landing(
        area,
        buf,
        title.clone(),
        cover.clone(),
    );
    
    let description = playlist.description.clone().unwrap_or_default();
    let description = HTML_UNICODE.replace_all(&description, |captures: &regex::Captures| {
        match captures.name("decimal") {
            Some(decimal) => std::char::from_u32(u32::from_str_radix(decimal.as_str(), 10).unwrap()).unwrap().to_string(),
            None => std::char::from_u32(u32::from_str_radix(captures.name("hex").unwrap().as_str(), 16).unwrap()).unwrap().to_string(),
        }
    });

    let info_highlight = if let LandingSection::Context = section { COLORS.highlight } else { Style::default() };

    let info = [
        if under.height <= 3 {
            Paragraph::new(description).style(info_highlight)
        } else {
            Paragraph::new(description).style(info_highlight).wrap(Wrap { trim: true })
        },
        Paragraph::new(playlist.owner.name.clone().unwrap_or(playlist.owner.id.clone())).style(info_highlight).bold(),
        Paragraph::new(match playlist.public.unwrap_or_default() {
            true => Span::from("Public").style(info_highlight).cyan(),
            false => Span::from("Private").style(info_highlight).magenta(),
        }),
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
    
    // RENDER PLAYLIST ITEMS
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

            Line::from("<No Playlist Items>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => {
            let scrollable = data.limit() >= main.height as usize;
            let block = Block::default().padding(Padding::new(0, if scrollable { 2 } else { 0 }, 0, 1));

            let table_albums = data
                .items
                .iter()
                .map(|a| match &a.item {
                    Item::Track(track) => format_track(track),
                    Item::Episode(episode) => format_episode(episode),
                })
                .collect::<Table>()
                .block(block)
                .widths([
                    Constraint::Length(8),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Fill(2),
                ])
                .highlight_style(COLORS.highlight);

            match section {
                LandingSection::Content => StatefulWidget::render(table_albums, main, buf, &mut state.clone()),
                LandingSection::Context => Widget::render(table_albums, main, buf)
            }

            PaginationProgress {
                current: data.page(),
                total: data.max_page(),
            }
            .render(main, buf);

            if scrollable {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(data.items.len()).position(state.selected().unwrap_or(0));
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
