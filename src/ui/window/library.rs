use crate::state::{window::library::{LibraryState, FromSpotify, LibraryTab}, Loading};
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
            Style::default().yellow()
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
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(block.inner(area));

        let from_spotify = Layout::horizontal([Constraint::Percentage(25); 4]).split(layout[0]);
        FromSpotify::iter().enumerate().for_each(|(i, f)| {
            f.render(from_spotify[i], buf, self.selection.is_spotify_playlist() && self.selected_spotify_playlist == f);
        });

        Tabs::new(LibraryTab::iter().map(|t| Line::from(t.title()).centered()))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .padding(" ", " ")
            .block(Block::bordered().borders(Borders::TOP | Borders::BOTTOM))
            .select(self.selected_tab as usize)
            .style(if self.selection.is_tabs() {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .divider(symbols::DOT)
            .render(layout[1], buf);

        match self.selected_tab {
            LibraryTab::Playlists => unwrap_render_results(
                self.playlists.items.lock().unwrap().as_ref(),
                self.result_state.clone(),
                layout[2],
                buf
            ),
            LibraryTab::Artists => unwrap_render_results(
                self.artists.items.lock().unwrap().as_ref(),
                self.result_state.clone(),
                layout[2],
                buf
            ),
            LibraryTab::Albums => unwrap_render_results(
                self.albums.items.lock().unwrap().as_ref(),
                self.result_state.clone(),
                layout[2],
                buf
            ),
            LibraryTab::Shows => unwrap_render_results(
                self.shows.items.lock().unwrap().as_ref(),
                self.result_state.clone(),
                layout[2],
                buf
            ),
            LibraryTab::Audiobooks => unwrap_render_results(
                self.audiobooks.items.lock().unwrap().as_ref(),
                self.result_state.clone(),
                layout[2],
                buf
            ),
        }
    }
}

pub trait IntoWidgetWrapper<T: StatefulWidget> {
    fn into_widget_wrapper(&self) -> T;
}

fn unwrap_render_results<'a, W: StatefulWidget, T: IntoWidgetWrapper<W>>(loading: Loading<T>, mut state: W::State, area: Rect, buf: &mut Buffer)
{
    match loading {
        Loading::Loading => {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                .split(area);

            Line::from("Loading...")
                .centered()
                .render(vert[1], buf);
        },
        Loading::None => {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                .split(area);

            Line::from("<No Results>")
                .centered()
                .red()
                .render(vert[1], buf);
        },
        Loading::Some(items) => {
            let block = Block::default()
                .padding(Padding::new(1, 1, 1, 0));
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
            .highlight_style(Style::default().yellow());

        StatefulWidget::render(table, area, buf, state);
    }
}

impl<'a> StatefulWidget for Artists<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    }
}

impl<'a> StatefulWidget for Albums<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    }
}

impl<'a> StatefulWidget for Shows<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    }
}

impl<'a> StatefulWidget for Audiobooks<'a> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    }
}
