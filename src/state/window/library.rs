use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
};

use rspotify::model::{
    ArtistId, CursorBasedPage, FullArtist, Page, SavedAlbum, Show, SimplifiedPlaylist,
};
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Play},
    state::{InnerState, Loadable},
    Error,
};

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    strum::EnumIter,
    strum::EnumCount,
    strum::FromRepr,
)]
pub enum Category {
    #[default]
    MadeForYou,
    Playlists,
    Artists,
    Albums,
    Shows,
}

impl Category {
    pub fn tab(&self) -> &'static str {
        match self {
            Self::MadeForYou => "Made For You",
            Self::Playlists => "Playlists",
            Self::Artists => "Artists",
            Self::Albums => "Albums",
            Self::Shows => "Shows",
        }
    }

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
}

#[derive(Default, Debug, Clone)]
pub struct LibraryState {
    pub category: Category,
    pub item: usize,

    pub featured: Vec<SimplifiedPlaylist>,

    pub artists: Loadable<CursorBasedPage<FullArtist>>,
    pub prev_artist: Vec<ArtistId<'static>>,

    pub playlists: Loadable<Page<SimplifiedPlaylist>>,
    pub albums: Loadable<Page<SavedAlbum>>,
    pub shows: Loadable<Page<Show>>,

    /* TODO: tracks, episodes */
}

impl LibraryState {
    pub fn handle_action(&mut self, action: Action, sender: UnboundedSender<Action>) -> Result<(), Error> {
        match action {
            Action::Up => {
                self.item = self.item.saturating_sub(1);
            }
            Action::Down => {
                let total = match self.category {
                    Category::MadeForYou => self.featured.len() as u32,
                    Category::Playlists => {
                        if let Loadable::Some(pl) = self.playlists.as_ref() {
                            pl.limit.saturating_sub(1)
                        } else {
                            0
                        }
                    }
                    Category::Artists => {
                        if let Loadable::Some(artists) = self.artists.as_ref() {
                            artists.limit.saturating_sub(1)
                        } else {
                            0
                        }
                    }
                    Category::Albums => {
                        if let Loadable::Some(albums) = self.albums.as_ref() {
                            albums.limit.saturating_sub(1)
                        } else {
                            0
                        }
                    }
                    Category::Shows => {
                        if let Loadable::Some(shows) = self.shows.as_ref() {
                            shows.limit.saturating_sub(1)
                        } else {
                            0
                        }
                    }
                };
                self.item = (self.item + 1).min(total as usize);
            }
            Action::Tab => {
                self.item = 0;
                self.category.next()
            }
            Action::BackTab => {
                self.item = 0;
                self.category.prev()
            }
            Action::Select => {
                match self.category {
                    Category::MadeForYou => {
                        let playlist = self.featured.get(self.item).ok_or(Error::custom("failed to select item from list 'Made For You'"))?;
                        sender.send(Action::Play(Play::Context(playlist.id.clone().into(), None, None)))?
                    },
                    Category::Playlists => if let Loadable::Some(playlists) = self.playlists.as_ref() {
                        let playlist = playlists.items.get(self.item).ok_or(Error::custom("failed to select item from list 'Playlists'"))?;
                        sender.send(Action::Play(Play::Context(playlist.id.clone().into(), None, None)))?
                    },
                    Category::Artists => if let Loadable::Some(artists) = self.artists.as_ref() {
                        let artist = artists.items.get(self.item).ok_or(Error::custom("failed to select item from list 'artists'"))?;
                        sender.send(Action::Play(Play::Context(artist.id.clone().into(), None, None)))?
                    },
                    Category::Albums => if let Loadable::Some(albums) = self.albums.as_ref() {
                        let album = albums.items.get(self.item).ok_or(Error::custom("failed to select item from list 'albums'"))?;
                        sender.send(Action::Play(Play::Context(album.album.id.clone().into(), None, None)))?
                    },
                    Category::Shows => if let Loadable::Some(shows) = self.shows.as_ref() {
                        let show = shows.items.get(self.item).ok_or(Error::custom("failed to select item from list 'shows'"))?;
                        sender.send(Action::Play(Play::Context(show.show.id.clone().into(), None, None)))?
                    },
                } 
            }
            Action::NextPage => {}
            Action::PreviousPage => {}
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
        Self::category(state, area, buf);
    }
}

impl Library {
    pub fn featured(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        let mut longest_owner = 0;
        let rows = lib 
            .featured
            .iter()
            .map(|p| {
                let owner_len = p.owner.display_name.as_deref().unwrap_or_default().len();
                if owner_len > longest_owner {
                    longest_owner = owner_len;
                }

                Row::new(vec![
                    Cell::from(p.name.clone()),
                    Cell::from(p.owner.display_name.clone().unwrap_or_default()),
                    Cell::from(if p.public.unwrap_or_default() {
                        "public"
                    } else {
                        "private"
                    }),
                ])
            })
            .collect::<Vec<Row>>();

        let widths = [
            Constraint::Fill(1),
            Constraint::Length(longest_owner as u16),
            Constraint::Length(7),
        ];

        let table = Table::new(rows, widths).highlight_style(Style::default().yellow());

        let mut ts = TableState::default().with_selected(lib.item);

        let block = Block::new().padding(Padding::horizontal(3));
        StatefulWidget::render(table, block.inner(area), buf, &mut ts);
    }

    pub fn category(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(area);

        let category = state.library.lock().unwrap().category;

        let tabs_block = Block::new()
            .padding(Padding::horizontal(1))
            .borders(Borders::BOTTOM);
        (&tabs_block).render(layout[0], buf);
        let tabs =
            Layout::horizontal([Constraint::Ratio(1, Category::COUNT as u32); Category::COUNT])
                .split(tabs_block.inner(layout[0]));

        for (i, tab) in Category::iter().enumerate() {
            Line::from(format!(" {} ", tab.tab()))
                .style(if i == category as usize {
                    Style::default().dark_gray().on_yellow()
                } else {
                    Style::default()
                })
                .centered()
                .render(tabs[i], buf);
        }

        match category {
            Category::MadeForYou => Self::featured(state, layout[1], buf),
            Category::Playlists => Self::playlists(state, layout[1], buf),
            Category::Artists => Self::artists(state, layout[1], buf),
            Category::Albums => Self::albums(state, layout[1], buf),
            Category::Shows => Self::shows(state, layout[1], buf),
        }
    }

    pub fn unwrap_page<'a, T>(
        loadable: &'a Loadable<T>,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> Option<&'a T> {
        match loadable.as_ref() {
            Loadable::None => {
                let vert = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(area);
                let layout = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(14),
                    Constraint::Fill(1),
                ])
                .split(vert[1])[1];

                let block = Block::new()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::all());
                (&block).render(layout, buf);

                Line::from("No Results").render(block.inner(layout), buf);
            }
            Loadable::Loading => {
                let vert = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(area);
                let layout = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(14),
                    Constraint::Fill(1),
                ])
                .split(vert[1])[1];

                Line::from("Loading...")
                    .cyan()
                    .centered()
                    .render(layout, buf);
            }
            Loadable::Some(v) => return Some(v),
        }
        None
    }

    pub fn playlists(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = Self::unwrap_page(&lib.playlists, area, buf) {
            let pages = (page.total as f32 / page.limit as f32).ceil() as u32;
            let current = if page.offset == 0 {
                1
            } else {
                page.offset / page.limit
            }
            .max(1);

            let layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(if pages > 1 { 2 } else { 0 }),
            ])
            .split(area);

            let rows = page
                .items
                .iter()
                .map(|p| {
                    Row::new(vec![
                        Cell::from(p.name.clone()),
                        Cell::from(p.owner.display_name.clone().unwrap_or_default()),
                        Cell::from(if p.public.unwrap_or_default() {
                            "public"
                        } else {
                            "private"
                        }),
                    ])
                })
                .collect::<Vec<Row>>();

            let widths = [
                Constraint::Fill(1),
                Constraint::Length(
                    page.items
                        .iter()
                        .map(|p| p.owner.display_name.as_deref().unwrap_or_default().len() as u16)
                        .max()
                        .unwrap_or(1)
                        .min(u16::MAX),
                ),
                Constraint::Length(7),
            ];

            let table = Table::new(rows, widths).highlight_style(Style::default().yellow());
            let mut ts = TableState::default().with_selected(lib.item);

            let block = Block::new().padding(Padding::horizontal(3));
            StatefulWidget::render(table, block.inner(layout[0]), buf, &mut ts);

            if pages > 1 {
                let page_block = Block::new().padding(Padding::top(1));
                Line::from(vec![
                    Span::from((0..current).map(|_| '•').collect::<String>()).green(),
                    Span::from((0..pages - current).map(|_| '•').collect::<String>()).dark_gray(),
                ])
                .centered()
                .render(page_block.inner(layout[1]), buf)
            }

            if layout[0].height < page.items.len() as u16 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(page.items.len()).position(lib.item);
                StatefulWidget::render(
                    scrollbar,
                    layout[0].inner(Margin {
                        vertical: 0,
                        horizontal: 1,
                    }),
                    buf,
                    &mut scrollbar_state,
                );
            }
        }
    }

    pub fn artists(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = Self::unwrap_page(&lib.artists, area, buf) {
            let pages = (page.total.unwrap_or_default() as f32 / page.limit as f32).ceil() as usize;
            let current = lib.prev_artist.len() + 1;

            let layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(if pages > 1 { 2 } else { 0 }),
            ])
            .split(area);

            let genres = page
                .items
                .iter()
                .map(|a| a.genres.join(", "))
                .collect::<Vec<String>>();
            let rows = page
                .items
                .iter()
                .zip(genres.iter())
                .map(|(a, genre)| {
                    Row::new(vec![Cell::from(a.name.clone()), Cell::from(genre.clone())])
                })
                .collect::<Vec<Row>>();

            let widths = [
                Constraint::Fill(1),
                Constraint::Length(genres.iter().map(|g| g.len()).max().unwrap_or_default() as u16),
            ];

            let table = Table::new(rows, widths).highlight_style(Style::default().yellow());

            let mut ts = TableState::default().with_selected(lib.item);

            let block = Block::new().padding(Padding::horizontal(3));
            StatefulWidget::render(table, block.inner(layout[0]), buf, &mut ts);

            if pages > 1 {
                let page_block = Block::new().padding(Padding::top(1));
                Line::from(vec![
                    Span::from((0..current).map(|_| '•').collect::<String>()).green(),
                    Span::from((0..pages - current).map(|_| '•').collect::<String>()).dark_gray(),
                ])
                .centered()
                .render(page_block.inner(layout[1]), buf)
            }

            if layout[0].height < page.items.len() as u16 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(page.items.len()).position(lib.item);
                StatefulWidget::render(
                    scrollbar,
                    layout[0].inner(Margin {
                        vertical: 0,
                        horizontal: 1,
                    }),
                    buf,
                    &mut scrollbar_state,
                );
            }
        }
    }

    pub fn albums(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = Self::unwrap_page(&lib.albums, area, buf) {
            let pages = (page.total as f32 / page.limit as f32).ceil() as u32;
            let current = if page.offset == 0 {
                1
            } else {
                page.offset / page.limit
            }
            .max(1);

            let layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(if pages > 1 { 2 } else { 0 }),
            ])
            .split(area);

            let artists = page
                .items
                .iter()
                .map(|a| {
                    a.album
                        .artists
                        .iter()
                        .map(|a| a.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .collect::<Vec<String>>();
            let rows = page
                .items
                .iter()
                .zip(artists.iter())
                .map(|(a, artist)| {
                    Row::new(vec![
                        Cell::from(a.album.name.as_str()),
                        Cell::from(artist.as_str()),
                    ])
                })
                .collect::<Vec<Row>>();

            let widths = [
                Constraint::Fill(1),
                Constraint::Length(artists.iter().map(|a| a.len()).max().unwrap_or_default() as u16),
            ];

            let table = Table::new(rows, widths).highlight_style(Style::default().yellow());

            let mut ts = TableState::default().with_selected(lib.item);

            let block = Block::new().padding(Padding::horizontal(3));
            StatefulWidget::render(table, block.inner(layout[0]), buf, &mut ts);

            if pages > 1 {
                let page_block = Block::new().padding(Padding::top(1));
                Line::from(vec![
                    Span::from((0..current).map(|_| '•').collect::<String>()).green(),
                    Span::from((0..pages - current).map(|_| '•').collect::<String>()).dark_gray(),
                ])
                .centered()
                .render(page_block.inner(layout[1]), buf)
            }

            if layout[0].height < page.items.len() as u16 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(page.items.len()).position(lib.item);
                StatefulWidget::render(
                    scrollbar,
                    layout[0].inner(Margin {
                        vertical: 0,
                        horizontal: 1,
                    }),
                    buf,
                    &mut scrollbar_state,
                );
            }
        }
    }

    pub fn shows(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = Self::unwrap_page(&lib.shows, area, buf) {
            let pages = (page.total as f32 / page.limit as f32).ceil() as u32;
            let current = if page.offset == 0 {
                1
            } else {
                page.offset / page.limit
            }
            .max(1);

            let layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(if pages > 1 { 2 } else { 0 }),
            ])
            .split(area);

            let mut longest_publisher = 0;
            let rows = page
                .items
                .iter()
                .map(|ctx| {
                    if ctx.show.publisher.len() > longest_publisher {
                        longest_publisher = ctx.show.publisher.len();
                    }

                    Row::new(vec![
                        Cell::from(ctx.show.name.as_str()),
                        Cell::from(if ctx.show.explicit { "explicit" } else { "" }).red(),
                        Cell::from(ctx.show.publisher.as_str()),
                    ])
                })
                .collect::<Vec<Row>>();

            let widths = [
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Length(longest_publisher as u16),
            ];

            let table = Table::new(rows, widths).highlight_style(Style::default().yellow());

            let mut ts = TableState::default().with_selected(lib.item);

            let block = Block::new().padding(Padding::horizontal(3));
            StatefulWidget::render(table, block.inner(layout[0]), buf, &mut ts);

            if pages > 1 {
                let page_block = Block::new().padding(Padding::top(1));
                Line::from(vec![
                    Span::from((0..current).map(|_| '•').collect::<String>()).green(),
                    Span::from((0..pages - current).map(|_| '•').collect::<String>()).dark_gray(),
                ])
                .centered()
                .render(page_block.inner(layout[1]), buf)
            }

            if layout[0].height < page.items.len() as u16 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(page.items.len()).position(lib.item);
                StatefulWidget::render(
                    scrollbar,
                    layout[0].inner(Margin {
                        vertical: 0,
                        horizontal: 1,
                    }),
                    buf,
                    &mut scrollbar_state,
                );
            }
        }
    }
}
