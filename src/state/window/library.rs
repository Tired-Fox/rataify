use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListState, Padding, StatefulWidget, Widget},
};

use strum::{EnumCount, VariantNames};

use crate::state::InnerState;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Section {
    #[default]
    Featured,
    Categories,
}

#[derive(
    Default, Debug, Clone, Copy, PartialEq, PartialOrd, strum::VariantNames, strum::EnumCount,
)]
pub enum Category {
    #[default]
    Playlists,
    Artists,
    Albums,
    Shows,
    Audiobooks,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LibraryState {
    pub section: Section,
    pub category: Category,
    pub featured: usize,
}

pub struct Library;

impl StatefulWidget for Library {
    type State = InnerState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let sections =
            Layout::horizontal([Constraint::Length(18), Constraint::Fill(1)]).split(area);

        Self::featured(state, sections[0], buf);
        Self::category(state, sections[1], buf);
    }
}

impl Library {
    pub fn featured(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let (section, featured) = {
            let lib = state.library.lock().unwrap();
            (lib.section, lib.featured)
        };

        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::RIGHT)
            .border_style(Style::default().dark_gray());
        (&block).render(area, buf);

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(block.inner(area));

        Line::from(vec![Span::from(" Made For You ").style(
            if Section::Featured == section {
                Style::default().dark_gray().on_yellow()
            } else {
                Style::default()
            },
        )])
        .centered()
        .render(layout[0], buf);

        let mut lines = Vec::new();

        if state.release_radar.lock().unwrap().is_some() {
            lines.push("Release Radar".to_string());
        }

        if state.discover_weekly.lock().unwrap().is_some() {
            lines.push("Discover Weekly".to_string());
        }

        for mix in state.daily_mixes.lock().unwrap().iter() {
            lines.push(mix.name.clone());
        }

        let list = List::new(lines).highlight_style(Style::default().yellow());

        let mut selected = if Section::Featured == section {
            ListState::default().with_selected(Some(featured))
        } else {
            ListState::default()
        };

        StatefulWidget::render(list, layout[2], buf, &mut selected);
    }

    pub fn category(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let (section, category) = {
            let lib = state.library.lock().unwrap();
            (lib.section, lib.category)
        };

        let tabs =
            Layout::horizontal([Constraint::Ratio(1, Category::COUNT as u32); Category::COUNT])
                .split(layout[0]);

        for (i, tab) in Category::VARIANTS.iter().enumerate() {
            Line::from(format!(" {} ", tab))
                .style(if section == Section::Categories && i == category as usize {
                    Style::default().dark_gray().on_yellow()
                } else {
                    Style::default()
                })
                .centered()
                .render(tabs[i], buf);
        }

        // TODO Paginated items
    }
}
