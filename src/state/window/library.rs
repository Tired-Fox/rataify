use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Cell, Padding, Row, StatefulWidget, Table, TableState, Widget},
};

use rspotify::{
    clients::OAuthClient,
    model::{ArtistId, Page},
    AuthCodePkceSpotify,
};
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    action::{Action, Open}, app::ContextSender, state::{model::{Album, Artist, Playlist, Show}, ActionList, InnerState, Loadable}, ConvertPage, Error
};

use super::Paginatable;

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
    Shows,
    Artists,
    Albums,
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

    pub featured: Vec<Playlist>,

    pub artists: Loadable<Page<Artist>>,
    pub prev_artist: Vec<Option<ArtistId<'static>>>,

    pub playlists: Loadable<Page<Playlist>>,
    pub albums: Loadable<Page<Album>>,
    pub shows: Loadable<Page<Show>>,
}

impl LibraryState {
    pub fn handle_action(
        &mut self,
        action: Action,
        spotify: &AuthCodePkceSpotify,
        state: &InnerState,
        sender: ContextSender,
    ) -> Result<(), Error> {
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
                        let playlist = self.featured.get(self.item).ok_or(Error::custom(
                            "failed to select item from list 'Made For You'",
                        ))?;
                        sender.send_action(Action::Open(Open::actions(playlist.action_list(true))))?
                    }
                    Category::Playlists => {
                        if let Loadable::Some(playlists) = self.playlists.as_ref() {
                            let playlist = playlists.items.get(self.item).ok_or(Error::custom(
                                "failed to select item from list 'Playlists'",
                            ))?;
                            sender.send_action(Action::Open(Open::actions(playlist.action_list(true))))?
                        }
                    }
                    Category::Artists => {
                        if let Loadable::Some(artists) = self.artists.as_ref() {
                            let artist = artists.items.get(self.item).ok_or(Error::custom(
                                "failed to select item from list 'artists'",
                            ))?;
                            sender.send_action(Action::Open(Open::actions(artist.action_list(true))))?
                        }
                    }
                    Category::Albums => {
                        if let Loadable::Some(albums) = self.albums.as_ref() {
                            let album = albums
                                .items
                                .get(self.item)
                                .ok_or(Error::custom("failed to select item from list 'albums'"))?;
                            sender.send_action(Action::Open(Open::actions(album.action_list(true))))?
                        }
                    }
                    Category::Shows => {
                        if let Loadable::Some(shows) = self.shows.as_ref() {
                            let show = shows
                                .items
                                .get(self.item)
                                .ok_or(Error::custom("failed to select item from list 'shows'"))?;
                            sender.send_action(Action::Open(Open::actions(show.action_list(true))))?
                        }
                    }
                }
            }
            Action::NextPage => {
                let spot = spotify.clone();
                let lib = state.library.clone();
                let c = sender.clone();
                match self.category {
                    Category::MadeForYou => {}
                    Category::Playlists => {
                        if let Loadable::Some(playlists) = self.playlists.clone() {
                            let offset = playlists.offset + playlists.limit;
                            if playlists.next.is_some() && offset < playlists.total {
                                self.playlists.load();
                                tokio::spawn(async move {
                                    match spot
                                        .current_user_playlists_manual(Some(20), Some(offset))
                                        .await
                                    {
                                        Ok(playlists) => lib
                                            .lock()
                                            .unwrap()
                                            .playlists
                                            .replace(playlists.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().playlists.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                    Category::Artists => {
                        if let Loadable::Some(artists) = self.artists.as_ref() {
                            if let Some(next) = artists.next.clone() {
                                if self.prev_artist.is_empty() {
                                    self.prev_artist.push(None);
                                } else {
                                    self.prev_artist
                                        .push(Some(artists.items.last().unwrap().id.clone()));
                                }
                                self.artists.load();
                                tokio::spawn(async move {
                                    match spot
                                        .current_user_followed_artists(
                                            Some(next.as_ref()),
                                            Some(20),
                                        )
                                        .await
                                    {
                                        Ok(artists) => lib
                                            .lock()
                                            .unwrap()
                                            .artists
                                            .replace(artists.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().artists.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                    Category::Albums => {
                        if let Loadable::Some(albums) = self.albums.as_ref() {
                            let offset = albums.offset + albums.limit;
                            if albums.next.is_some() && offset < albums.total {
                                self.albums.load();
                                tokio::spawn(async move {
                                    match spot
                                        .current_user_saved_albums_manual(
                                            None,
                                            Some(20),
                                            Some(offset),
                                        )
                                        .await
                                    {
                                        Ok(albums) => lib.lock().unwrap().albums.replace(albums.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().albums.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                    Category::Shows => {
                        if let Loadable::Some(shows) = self.shows.as_ref() {
                            let offset = shows.offset + shows.limit;
                            if shows.next.is_some() && offset < shows.total {
                                self.shows.load();
                                tokio::spawn(async move {
                                    match spot.get_saved_show_manual(Some(20), Some(offset)).await {
                                        Ok(shows) => lib.lock().unwrap().shows.replace(shows.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().shows.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                }
            }
            Action::PreviousPage => {
                let spot = spotify.clone();
                let lib = state.library.clone();
                let c = sender.clone();
                match self.category {
                    Category::MadeForYou => {}
                    Category::Playlists => {
                        if let Loadable::Some(playlists) = self.playlists.clone() {
                            let offset = playlists.offset.saturating_sub(playlists.limit);
                            if playlists.previous.is_some() && offset < playlists.offset {
                                self.playlists.load();
                                tokio::spawn(async move {
                                    match spot
                                        .current_user_playlists_manual(Some(20), Some(offset))
                                        .await
                                    {
                                        Ok(playlists) => lib
                                            .lock()
                                            .unwrap()
                                            .playlists
                                            .replace(playlists.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().playlists.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                    Category::Artists => {
                        if self.artists.is_some() {
                            if let Some(after) =
                                self.prev_artist.pop().map(|v| v.map(|v| v.to_string()))
                            {
                                self.artists.load();
                                tokio::spawn(async move {
                                    match spot
                                        .current_user_followed_artists(
                                            if let Some(after) = after.as_deref() {
                                                Some(after)
                                            } else {
                                                None
                                            },
                                            Some(20),
                                        )
                                        .await
                                    {
                                        Ok(artists) => lib
                                            .lock()
                                            .unwrap()
                                            .artists
                                            .replace(artists.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().artists.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                    Category::Albums => {
                        if let Loadable::Some(albums) = self.albums.as_ref() {
                            let offset = albums.offset.saturating_sub(albums.limit);
                            if albums.previous.is_some() && offset < albums.offset {
                                self.albums.load();
                                tokio::spawn(async move {
                                    match spot
                                        .current_user_saved_albums_manual(
                                            None,
                                            Some(20),
                                            Some(offset),
                                        )
                                        .await
                                    {
                                        Ok(albums) => lib.lock().unwrap().albums.replace(albums.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().albums.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                    Category::Shows => {
                        if let Loadable::Some(shows) = self.shows.as_ref() {
                            let offset = shows.offset.saturating_sub(shows.limit);
                            if shows.previous.is_some() && offset < shows.offset {
                                self.shows.load();
                                tokio::spawn(async move {
                                    match spot.get_saved_show_manual(Some(20), Some(offset)).await {
                                        Ok(shows) => lib.lock().unwrap().shows.replace(shows.convert_page()),
                                        Err(err) => {
                                            c.send_error(err.into()).unwrap();
                                            lib.lock().unwrap().shows.take()
                                        }
                                    };
                                });
                            }
                        }
                    }
                }
            }
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
                    }).magenta(),
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

        StatefulWidget::render(table, area, buf, &mut ts);
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

        let block = Block::default();

        match category {
            Category::MadeForYou => Self::featured(state, block.inner(layout[1]), buf),
            Category::Playlists => Self::playlists(state, block.inner(layout[1]), buf),
            Category::Artists => Self::artists(state, block.inner(layout[1]), buf),
            Category::Albums => Self::albums(state, block.inner(layout[1]), buf),
            Category::Shows => Self::shows(state, block.inner(layout[1]), buf),
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
        if let Some(page) = lib.playlists.render_unwrap(area, buf) {
            let mut state = TableState::default().with_selected(lib.item);
            page.paginated(None, lib.item).render(area, buf, &mut state);
        }
    }

    pub fn artists(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = lib.artists.render_unwrap(area, buf) {
            let mut state = TableState::default().with_selected(lib.item);
            page.paginated(Some(lib.prev_artist.len() as u32 * page.limit), lib.item)
                .render(area, buf, &mut state);
        }
    }

    pub fn albums(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = lib.albums.render_unwrap(area, buf) {
            let mut state = TableState::default().with_selected(lib.item);
            page.paginated(None, lib.item).render(area, buf, &mut state);
        }
    }

    pub fn shows(
        state: &mut InnerState,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let lib = state.library.lock().unwrap();
        if let Some(page) = lib.shows.render_unwrap(area, buf) {
            let mut state = TableState::default().with_selected(lib.item);
            page.paginated(None, lib.item).render(area, buf, &mut state);
        }
    }
}
