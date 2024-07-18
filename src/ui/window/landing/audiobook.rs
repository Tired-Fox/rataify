use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap
    },
};
use tupy::api::response::{Audiobook, Chapters, Paged};

use crate::{
    state::{
        window::{
            landing::{Cover, LandingSection},
            Pages,
        },
        Loading,
    },
    ui::{components::OpenInSpotify, format_duration, PaginationProgress, COLORS},
    Locked, Shared,
};

use super::{render_landing, HTML_UNICODE};

#[allow(clippy::too_many_arguments)]
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    audiobook: &Audiobook,
    pages: &Pages<Chapters, Chapters>,
    state: &TableState,
    section: &LandingSection,
    cover: &mut Shared<Locked<Loading<Cover>>>,
) {
    let title = format!("[Audiobook: {}]", audiobook.name);
    let (under, main) = render_landing(
        area,
        buf,
        title.clone(),
        cover.clone(),
    );
    
    // desc, authors, narrators, publisher
    let description = audiobook.description.clone();
    let description = HTML_UNICODE.replace_all(&description, |captures: &regex::Captures| {
        match captures.name("decimal") {
            Some(decimal) => std::char::from_u32(u32::from_str_radix(decimal.as_str(), 10).unwrap()).unwrap().to_string(),
            None => std::char::from_u32(u32::from_str_radix(captures.name("hex").unwrap().as_str(), 16).unwrap()).unwrap().to_string(),
        }
    });

    let info_highlight = if let LandingSection::Context = section { COLORS.highlight } else { Style::default() };

    let info = [
        Paragraph::new(description).style(info_highlight).wrap(Wrap { trim: true })
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

            Line::from("<No Audiobook Chapters>")
                .centered()
                .red()
                .render(vert, buf);
        }
        Some(Loading::Some(data)) => {
            let scrollable = data.limit() >= main.height as usize;
            let block = Block::default().padding(Padding::new(0, if scrollable { 2 } else { 0 }, 0, 1));

            let table_chapters = data.items
                .iter()
                .map(|e| {
                    Row::new(vec![
                        // duration, chapter, name, finished
                        Cell::from(format_duration(e.duration)).style(COLORS.duration),
                        Cell::from(format!("Chapter {}", e.chapter_number + 1)).style(COLORS.chapter_number),
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
                    Constraint::Length(8 + format!("{}", data.total).len() as u16),
                    Constraint::Fill(1),
                ])
                .block(block)
                .highlight_style(COLORS.highlight);

            match section {
                LandingSection::Content => StatefulWidget::render(table_chapters, main, buf, &mut state.clone()),
                LandingSection::Context => Widget::render(table_chapters, main, buf)
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
