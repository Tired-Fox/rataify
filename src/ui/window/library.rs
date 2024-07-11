use crate::{state::{window::library::{FromSpotify, LibraryState, LibraryTab}, Loading}, ui::COLORS};
use strum::IntoEnumIterator;
use tupy::api::response::{PagedPlaylists, FollowedArtists, SavedAlbums, SavedShows, SavedAudiobooks};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{self, border},
    text::{Line, Span},
    widgets::{
        block::{Position, Title},
        Block, Borders, Row, Table, Tabs, StatefulWidget, Widget,
        TableState, Padding
    },
};

use crate::ui::PaginationProgress;

impl FromSpotify {
    fn render(self, area: Rect, buf: &mut Buffer, selected: bool) {
        let (title, lines) = match &self {
            FromSpotify::ReleaseRadar => (self.title(), [
                Line::default(),
                Line::default(),
                Line::default(),
                Line::from("RR").centered(),
                Line::default(),
                Line::default(),
                Line::default(),
            ]),
            FromSpotify::DiscoverWeekly => (self.title(), [
                Line::default(),
                Line::default(),
                Line::default(),
                Line::from("DW").centered(),
                Line::default(),
                Line::default(),
                Line::default(),
            ]),
            FromSpotify::LikedSongs => (self.title(), [
                Line::default(),
                Line::default(),
                Line::default(),
                Line::from("♥").centered().red(),
                Line::default(),
                Line::default(),
                Line::default(),
            ]),
            FromSpotify::MyEpisodes => (self.title(), [
                Line::default(),
                Line::default(),
                Line::default(),
                Line::from("✓").centered().green(),
                Line::default(),
                Line::default(),
                Line::default(),
            ]),
        };

        Art::new(title, lines, selected)
            .render(area, buf);
    }
}

struct Art<'a> {
    image: [Line<'a>; 7],
    title: String,
    selected: bool,
}
impl<'a> Art<'a> {
    fn new<T>(title: T, image: [Line<'a>; 7], selected: bool) -> Self
    where
        T: AsRef<str>,
    {
        Self { 
            image,
            title: title.as_ref().to_string(),
            selected,
        }
    }
}

impl<'a> Widget for Art<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 5 x 5 design
        let vert = Layout::vertical([Constraint::Length(9), Constraint::Length(1)])
            .split(area);

        let image_area = Layout::horizontal([Constraint::Fill(1), Constraint::Length(18), Constraint::Fill(1)])
            .split(vert[0])[1];

        let style = if self.selected {
            COLORS.highlight
        } else {
            Style::default()
        };

        let block = Block::bordered()
            .borders(Borders::ALL)
            .style(style)
            .border_set(border::ROUNDED);
        (&block).render(image_area, buf);

        let inner = Layout::vertical([Constraint::Length(1);7]).split(block.inner(image_area));


        for (i, line) in self.image.iter().enumerate().take(7) {
            line.render(inner[i], buf);
        }

        Line::from(self.title)
            .centered()
            .style(style)
            .render(vert[1], buf);
    }
}

impl Widget for &LibraryState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .borders(Borders::all())
            .title(
                Title::from("[Library]")
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::ROUNDED);

        (&block).render(area, buf);

        let layout = Layout::vertical([
            Constraint::Length(11),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .split(block.inner(area));

        let from_spotify = Layout::horizontal([Constraint::Percentage(25); 4]).split(layout[0]);
        FromSpotify::iter().enumerate().for_each(|(i, f)| {
            f.render(from_spotify[i], buf, self.selection.is_spotify_playlist() && self.selected_spotify_playlist == f);
        });

        Tabs::new(LibraryTab::iter().map(|t| Line::from(t.title()).centered()))
            .highlight_style(COLORS.highlight)
            .padding(" ", " ")
            //.block(Block::bordered().borders(Borders::TOP | Borders::BOTTOM))
            .select(self.selected_tab as usize)
            .divider(symbols::DOT)
            .render(layout[1], buf);

        match self.selected_tab {
            LibraryTab::Playlists => {
                let results = self.playlists.items.lock().unwrap();
                let items = results.as_ref().map(|a| a.as_ref());
                if items.is_some() && items.as_ref().unwrap().is_some() {
                    let area = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(layout[2])[1];
                    let pages = self.playlists.pages.lock().unwrap();
                    if pages.1 > 1 {
                        PaginationProgress {
                            current: pages.0,
                            total: pages.1,
                        }
                            .render(area, buf);
                    }
                }
                unwrap_render_results(
                    items,
                    self.result_state.clone(),
                    layout[2],
                    buf
                )
            },
            LibraryTab::Artists => {
                let results = self.artists.items.lock().unwrap();
                let items = results.as_ref().map(|a| a.as_ref());
                if items.is_some() && items.as_ref().unwrap().is_some() {
                    let area = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(layout[2])[1];
                    let pages = self.artists.pages.lock().unwrap();
                    if pages.1 > 1 {
                        PaginationProgress {
                            current: pages.0,
                            total: pages.1,
                        }
                            .render(area, buf);
                    }
                }
                unwrap_render_results(
                    items,
                    self.result_state.clone(),
                    layout[2],
                    buf
                )
            },
            LibraryTab::Albums => {
                let results = self.albums.items.lock().unwrap();
                let items = results.as_ref().map(|a| a.as_ref());
                if items.is_some() && items.as_ref().unwrap().is_some() {
                    let area = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(layout[2])[1];
                    let pages = self.albums.pages.lock().unwrap();
                    if pages.1 > 1 {
                        PaginationProgress {
                            current: pages.0,
                            total: pages.1,
                        }
                            .render(area, buf);
                    }
                }
                unwrap_render_results(
                    items,
                    self.result_state.clone(),
                    layout[2],
                    buf
                )
            },
            LibraryTab::Shows => {
                let results = self.shows.items.lock().unwrap();
                let items = results.as_ref().map(|a| a.as_ref());
                if items.is_some() && items.as_ref().unwrap().is_some() {
                    let area = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(layout[2])[1];
                    let pages = self.shows.pages.lock().unwrap();
                    if pages.1 > 1 {
                        PaginationProgress {
                            current: pages.0,
                            total: pages.1,
                        }
                            .render(area, buf);
                    }
                }
                unwrap_render_results(
                    items,
                    self.result_state.clone(),
                    layout[2],
                    buf
                )
            },
            LibraryTab::Audiobooks => {
                let results = self.audiobooks.items.lock().unwrap();
                let items = results.as_ref().map(|a| a.as_ref());
                if items.is_some() && items.as_ref().unwrap().is_some() {
                    let area = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(layout[2])[1];
                    let pages = self.audiobooks.pages.lock().unwrap();
                    if pages.1 > 1 {
                        PaginationProgress {
                            current: pages.0,
                            total: pages.1,
                        }
                            .render(area, buf);
                    }
                }
                unwrap_render_results(
                    items,
                    self.result_state.clone(),
                    layout[2],
                    buf
                )
            },
        }
    }
}

pub trait IntoWidgetWrapper<T: StatefulWidget> {
    fn into_widget_wrapper(&self) -> T;
}

fn unwrap_render_results<'a, W: StatefulWidget, T: IntoWidgetWrapper<W>>(loading: Option<Loading<T>>, mut state: W::State, area: Rect, buf: &mut Buffer)
{
    match loading {
        Some(Loading::Loading) => {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                .split(area);

            Line::from("Loading...")
                .centered()
                .render(vert[1], buf);
        },
        None | Some(Loading::None) => {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                .split(area);

            Line::from("<No Results>")
                .centered()
                .red()
                .render(vert[1], buf);
        },
        Some(Loading::Some(items)) => {
            let block = Block::default()
                .padding(Padding::horizontal(1));
            let area = block.inner(area);
            StatefulWidget::render(items.into_widget_wrapper(), area, buf, &mut state)
        },
    }
}

/* WRAPPERS */

pub struct Playlists<'a>(&'a PagedPlaylists);
impl<'a> IntoWidgetWrapper<Playlists<'a>> for &'a PagedPlaylists {
    fn into_widget_wrapper(&self) -> Playlists<'a> {
        Playlists(self)
    }
}

pub struct Artists<'a>(&'a FollowedArtists);
impl<'a> IntoWidgetWrapper<Artists<'a>> for &'a FollowedArtists {
    fn into_widget_wrapper(&self) -> Artists<'a> {
        Artists(self)
    }
}

pub struct Albums<'a>(&'a SavedAlbums);
impl<'a> IntoWidgetWrapper<Albums<'a>> for &'a SavedAlbums {
    fn into_widget_wrapper(&self) -> Albums<'a> {
        Albums(self)
    }
}

pub struct Shows<'a>(&'a SavedShows);
impl<'a> IntoWidgetWrapper<Shows<'a>> for &'a SavedShows {
    fn into_widget_wrapper(&self) -> Shows<'a> {
        Shows(self)
    }
}

pub struct Audiobooks<'a>(&'a SavedAudiobooks);
impl<'a> IntoWidgetWrapper<Audiobooks<'a>> for &'a SavedAudiobooks {
    fn into_widget_wrapper(&self) -> Audiobooks<'a> {
        Audiobooks(self)
    }
}

/* WIDGET IMPLEMENTATIONS */

impl<'a> StatefulWidget for Playlists<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // name ownder public or private
        let table = self.0.items.iter().map(|p| {
            Row::new(vec![
                Line::from(p.name.clone()),
                Line::from(p.owner.name.clone().unwrap_or(p.owner.id.clone())).right_aligned(),
                Line::from(if p.public.unwrap_or(false) { "Public".cyan() } else { "Private".red() })
            ])
        })
            .collect::<Table>()
            .column_spacing(1)
            .highlight_style(COLORS.highlight);

        StatefulWidget::render(table, area, buf, state);
    }
}

impl<'a> StatefulWidget for Artists<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = self.0.items.iter().map(|a| {
            Row::new(vec![
                Line::from(a.name.clone()),
                Line::from(a.genres.join(", ")).right_aligned(),
            ])
        })
            .collect::<Table>()
            .column_spacing(1)
            .highlight_style(COLORS.highlight);

        StatefulWidget::render(table, area, buf, state);
    }
}

impl<'a> StatefulWidget for Albums<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = self.0.items.iter().map(|a| {
            Row::new(vec![
                Line::from(a.album.name.clone()),
                Line::from(format!("{:?}", a.album.album_type)).right_aligned(),
                Line::from(a.album.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", "))
            ])
        })
            .collect::<Table>()
            .column_spacing(1)
            .highlight_style(COLORS.highlight);

        StatefulWidget::render(table, area, buf, state);
    }
}

impl<'a> StatefulWidget for Shows<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = self.0.items.iter().map(|a| {
            Row::new(vec![
                Line::from(a.show.name.clone()),
                Line::from(a.show.publisher.clone().unwrap_or_default()).right_aligned(),
                Line::from(format!("[{}]", a.show.total_episodes))
            ])
        })
            .collect::<Table>()
            .column_spacing(1)
            .highlight_style(COLORS.highlight);

        StatefulWidget::render(table, area, buf, state);
    }
}

impl<'a> StatefulWidget for Audiobooks<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = self.0.items.iter().map(|a| {
            Row::new(vec![
                Line::from(a.name.clone()),
                Line::from(a.publisher.clone()),
                Line::from(a.authors.join(", ")),
                Line::from(a.edition.clone()),
            ])
        })
            .collect::<Table>()
            .column_spacing(1)
            .highlight_style(COLORS.highlight);

        StatefulWidget::render(table, area, buf, state);
    }
}
