use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListState, Padding, Row, StatefulWidget, Table, TableState, Widget},
};

use rspotify::model::{
    ArtistId, CursorBasedPage, FullArtist, Page, SavedAlbum, Show, SimplifiedPlaylist
};
use strum::{EnumCount, VariantNames};

use crate::{action::Action, state::{InnerState, Loadable}, Error};

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Section {
    #[default]
    Featured,
    Categories,
}

impl Section {
    pub fn next(&mut self) {
        let next = match self {
            Self::Featured => Self::Categories,
            Self::Categories => Self::Featured,
        };
        let _ = std::mem::replace(self, next);
    }

    pub fn prev(&mut self) {
        let previous = match self {
            Self::Featured => Self::Categories,
            Self::Categories => Self::Featured,
        };
        let _ = std::mem::replace(self, previous);
    }
}

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    strum::VariantNames,
    strum::EnumCount,
    strum::FromRepr,
)]
pub enum Category {
    #[default]
    Playlists,
    Artists,
    Albums,
    Shows,
}

impl Category {
    pub fn replace(&mut self, category: Self) {
        let _ = std::mem::replace(self, category);
    }

    pub fn next(&mut self) {
        let next = Self::from_repr(((*self as usize) + 1) % Self::COUNT).unwrap_or_default();
        let _ = std::mem::replace(self, next);
    }

    pub fn prev(&mut self) {
        let mut prev = (*self as isize) - 1;
        if prev < 0 {
            prev = (Self::COUNT - 1) as isize;
        }

        let prev = Self::from_repr(prev as usize).unwrap_or(Self::Shows);
        let _ = std::mem::replace(self, prev);
    }

    pub fn is_last(&self) -> bool {
        *self as usize == (Self::COUNT - 1)
    }

    pub fn is_first(&self) -> bool {
        *self as usize == 0
    }
}

#[derive(Default, Debug, Clone)]
pub struct LibraryState {
    pub section: Section,
    pub category: Category,

    pub item: usize,
    pub featured: usize,

    pub artists: Loadable<CursorBasedPage<FullArtist>>,
    pub prev_artist: Vec<ArtistId<'static>>,

    pub playlists: Loadable<Page<SimplifiedPlaylist>>,
    pub albums: Loadable<Page<SavedAlbum>>,
    pub shows: Loadable<Page<Show>>,
    /* TODO: tracks, episodes */
}

impl LibraryState {
    pub fn handle_action(&mut self, action: Action, featured: usize) -> Result<(), Error> {
        match action {
            Action::Up => match self.section {
                Section::Featured => {
                    self.featured = self.featured.saturating_sub(1);
                },
                Section::Categories => {
                    self.item = self.item.saturating_sub(1);
                }
            },
            Action::Down => {
                match self.section {
                    Section::Featured => self.featured = (self.featured + 1).min(featured),
                    Section::Categories => {
                        let total = match self.category {
                            Category::Playlists => if let Loadable::Some(pl) = self.playlists.as_ref() {
                                pl.total.saturating_sub(1)
                            } else { 0 },
                            Category::Artists => if let Loadable::Some(artists) = self.artists.as_ref() { artists.total.unwrap_or_default().saturating_sub(1) } else { 0 },
                            Category::Albums => if let Loadable::Some(albums) = self.albums.as_ref() { albums.total.saturating_sub(1) } else { 0 },
                            Category::Shows => if let Loadable::Some(shows) = self.shows.as_ref() { shows.total.saturating_sub(1) } else { 0 },
                        };

                        self.item = (self.item + 1).min(total as usize);
                    }
                };
            }
            Action::Tab => {
                match self.section {
                    Section::Featured => { 
                        self.section.next();
                        self.category.replace(Category::Playlists);
                    },
                    Section::Categories => if self.category.is_last() {
                        self.section.next();
                    } else {
                        self.category.next()
                    }
                }
            },
            Action::BackTab => {
                match self.section {
                    Section::Featured => {
                        self.section.prev();
                        self.category.replace(Category::Shows);
                    },
                    Section::Categories => if self.category.is_first() {
                        self.section.prev();
                    } else {
                        self.category.prev()
                    }
                }
            },
            Action::Select => {},
            Action::NextPage => {},
            Action::PreviousPage => {},
            _ => {}
        }

        Ok(())
    }
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
        for playlist in state.featured.lock().unwrap().iter() {
            lines.push(playlist.name.clone());
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
                .style(
                    if section == Section::Categories && i == category as usize {
                        Style::default().dark_gray().on_yellow()
                    } else {
                        Style::default()
                    },
                )
                .centered()
                .render(tabs[i], buf);
        }

        match category {
            Category::Playlists => Self::playlists(state, section, layout[2], buf),
            Category::Artists => Self::artists(state, section, layout[2], buf),
            Category::Albums => Self::albums(state, section, layout[2], buf),
            Category::Shows => Self::shows(state, section, layout[2], buf),
        }
    }

    pub fn playlists(
        state: &mut InnerState,
        section: Section,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();

        match lib.playlists.as_ref() {
            Loadable::None => {
                let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).split(area);
                let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(14), Constraint::Fill(1)]).split(vert[1])[1];
                
                let block = Block::new()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::all());
                (&block).render(layout, buf);

                Line::from("No Results")
                    .render(block.inner(layout), buf);
            }
            Loadable::Loading => {
                let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).split(area);
                let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(14), Constraint::Fill(1)]).split(vert[1])[1];
                
                Line::from("Loading...")
                    .cyan()
                    .centered()
                    .render(layout, buf);
            }
            Loadable::Some(t) => {
                let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)]).split(area);
                
                let block = Block::new()
                    .padding(Padding::symmetric(4, 1));
                (&block).render(layout[0], buf);

                let rows = t.items.iter().map(|p| Row::new(vec![
                    Cell::from(p.name.clone()),
                    Cell::from(p.owner.display_name.clone().unwrap_or_default()),
                    Cell::from(if p.public.unwrap_or_default() { "public" } else { "private" }),
                ])).collect::<Vec<Row>>();

                let widths = [
                    Constraint::Fill(1),
                    Constraint::Length(t.items.iter().map(|p| p.owner.display_name.as_deref().unwrap_or_default().len() as u16).max().unwrap_or(1).min(u16::MAX)),
                    Constraint::Length(7),
                ];

                let table = Table::new(rows, widths)
                    .block(block)
                    .highlight_style(if section == Section::Categories { Style::default().yellow() } else { Style::default() });

                let mut ts = TableState::default();
                if section == Section::Categories {
                    ts = ts.with_selected(lib.item);
                }

                StatefulWidget::render(table, layout[0], buf, &mut ts);

                let pages = (t.total as f32/t.limit as f32).ceil() as u32;
                let current = if t.offset == 0 { 1 } else { t.offset / t.limit }.max(1);

                Line::from(vec![
                    Span::from((0..current).map(|_| '•').collect::<String>()).green(),
                    Span::from((0..pages-current).map(|_| '•').collect::<String>()).dark_gray(),
                ])
                    .centered()
                    .render(layout[2], buf)
            }
        }
    }

    pub fn artists(
        state: &mut InnerState,
        section: Section,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {

    }

    pub fn albums(
        state: &mut InnerState,
        section: Section,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {

    }

    pub fn shows(
        state: &mut InnerState,
        section: Section,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {

    }
}
