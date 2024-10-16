use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap
    },
};
use tupy::api::response::{Paged, Show, ShowEpisodes, SimplifiedEpisode};

use crate::{
    state::{
        window::{
            landing::{Cover, LandingSection}, MappedPages, Pages
        }, wrappers::Saved, Loading
    },
    ui::{format_duration, PaginationProgress, COLORS},
    Locked, Shared,
};

use super::{render_landing, HTML_UNICODE};

#[allow(clippy::too_many_arguments)]
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    show: &Show,
    pages: &MappedPages<Vec<Saved<SimplifiedEpisode>>, ShowEpisodes, ShowEpisodes>,
    state: &TableState,
    section: &LandingSection,
    cover: &mut Shared<Locked<Loading<Cover>>>,
) {
    let title = format!("[Show: {}]", show.name);
    let (under, main) = render_landing(
        area,
        buf,
        title.clone(),
        cover.clone(),
    );
    
    let description = show.description.clone().unwrap_or_default();
    let description = HTML_UNICODE.replace_all(&description, |captures: &regex::Captures| {
        match captures.name("decimal") {
            Some(decimal) => std::char::from_u32(u32::from_str_radix(decimal.as_str(), 10).unwrap()).unwrap().to_string(),
            None => std::char::from_u32(u32::from_str_radix(captures.name("hex").unwrap().as_str(), 16).unwrap()).unwrap().to_string(),
        }
    });

    let info_highlight = if let LandingSection::Context = section { COLORS.highlight } else { Style::default() };

    let info = [
        if under.height <= 2 {
            Paragraph::new(description).style(info_highlight)
        } else {
            Paragraph::new(description).style(info_highlight).wrap(Wrap { trim: true })
        },
        Paragraph::new(show.publisher.clone().unwrap_or_default()).style(info_highlight).bold(),
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
    
    // RENDER SHOW EPISODES
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

            Line::from("<No Show Items>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => {
            let page = pages.page.lock().unwrap();
            let scrollable = page.limit >= main.height as usize;
            let block = Block::default().padding(Padding::new(0, if scrollable { 2 } else { 0 }, 0, 1));

            let table_episodes = data
                .iter()
                .map(|e| {
                    Row::new(vec![
                        Cell::from(if e.saved { "♥" } else { "" }).style(COLORS.like),
                        Cell::from(format_duration(e.as_ref().duration)).style(COLORS.duration),
                        if e.as_ref().resume_point.fully_played {
                            Cell::from("✓").style(COLORS.finished)
                        } else {
                            Cell::default()
                        },
                        Cell::from(e.as_ref().name.clone()).style(COLORS.episode),
                    ])
                })
                .collect::<Table>()
                .widths([
                    Constraint::Length(2),
                    Constraint::Length(8),
                    Constraint::Length(1),
                    Constraint::Fill(2),
                ])
                .block(block)
                .highlight_style(if section.is_content() { COLORS.highlight } else { Style::default() });

            StatefulWidget::render(table_episodes, main, buf, &mut state.clone());

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
