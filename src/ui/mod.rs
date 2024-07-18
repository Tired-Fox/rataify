use lazy_static::lazy_static;
use crossterm::event::KeyEvent;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, symbols::{border, DOT}, text::{Line, Span}, widgets::{block::{Position, Title}, Block, Cell, Row, StatefulWidget, Widget}, style::{Color, Style, Stylize}};
use tupy::{api::{request::Play, response::{Context, Episode, PlaybackItem, SimplifiedAlbum, SimplifiedChapter, SimplifiedEpisode, SimplifiedTrack, Track}}, Duration};

pub mod modal;
pub mod window;
pub mod playback;
pub mod action;
pub mod components;

pub use action::{ActionLabel, Action, GoTo};

pub use playback::NoPlayback;

use crate::{state::{playback::{Item, PlaybackState}, Modal, State, Viewport, Window}, key};

use self::modal::goto::UiGoto;

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn centered_rect_limited(percent_x: u16, percent_y: u16, w: u16, h: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Min(w),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Min(h),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn format_duration(duration: Duration) -> String {
    if duration >= Duration::hours(1) {
        format!(
            "{:0>2}:{:0>2}:{:0>2}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    } else {
        format!(
            "{:0>2}:{:0>2}",
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    }
}

/// Icon | Duration | Name | By | Context
fn format_track<'l>(track: &Track) -> Row<'l> {
    Row::new(vec![
        Cell::from(Line::from(format_duration(track.duration)).right_aligned().style(COLORS.duration)),
        Cell::default(),
        Cell::from(track.name.clone()).cyan().style(COLORS.track),
        Cell::from(
            track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ).style(COLORS.artists),
    ])
}

/// Icon | Duration | Name | By | Context
fn format_episode<'l>(episode: &Episode) -> Row<'l> {
    let mut cells = vec![
        Cell::from(Line::from(format_duration(episode.duration)).right_aligned().style(COLORS.duration)),
        if episode.resume_point.fully_played {
            Cell::from("✓").style(COLORS.finished)
        } else {
            Cell::default()
        },
        Cell::from(episode.name.clone()).style(COLORS.episode)
    ];

    if let Some(show) = episode.show.as_ref() {
        cells.push(Cell::from(show.name.clone()).style(COLORS.context));
    } else {
        cells.push(Cell::default());
    }

    Row::new(cells)
}

/// Icon | Duration | Name | By | Context
fn format_track_saved<'l>(track: &Track, saved: bool) -> Row<'l> {
    Row::new(vec![
        Cell::from(if saved { "♥" } else { "" }).style(COLORS.like),
        Cell::from(Line::from(format_duration(track.duration)).right_aligned().style(COLORS.duration)),
        Cell::default(),
        Cell::from(track.name.clone()).style(COLORS.track),
        Cell::from(
            track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ).style(COLORS.artists),
    ])
}

/// Icon | Duration | Name | By | Context
fn format_episode_saved<'l>(episode: &Episode, saved: bool) -> Row<'l> {
    let mut cells = vec![
        Cell::from(if saved { "♥" } else { "" }).style(COLORS.like),
        Cell::from(Line::from(format_duration(episode.duration)).right_aligned().style(COLORS.duration)),
        if episode.resume_point.fully_played {
            Cell::from("✓").style(COLORS.finished)
        } else {
            Cell::default()
        },
        Cell::from(episode.name.clone()).style(COLORS.episode)
    ];

    if let Some(show) = episode.show.as_ref() {
        cells.push(Cell::from(show.name.clone()).style(COLORS.context));
    } else {
        cells.push(Cell::default());
    }

    Row::new(cells)
}

pub trait IntoActions {
    fn into_ui_actions(self, context: bool) -> Vec<(KeyEvent, Action, &'static str)>;
}

impl IntoActions for &SimplifiedAlbum {
    fn into_ui_actions(self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let mut actions = vec![
            (key!(Enter), Action::PlayContext(Play::album(self.uri.clone(), None, 0)), ActionLabel::Play),
            (key!('C'), Action::GoTo(GoTo::Album(self.uri.clone())), ActionLabel::GoToAlbum),
        ];

        if self.artists.len() > 1 {
            actions.push((key!('A'), Action::GoTo(
                GoTo::Artists(
                    self.artists.iter().map(|a| (a.uri.clone(), a.name.clone())).collect::<Vec<_>>()
                )
            ), ActionLabel::SelectArtist))
        }

        actions
    }
}

impl IntoActions for &Track {
    fn into_ui_actions(self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let mut actions = vec![
            (key!('p'),Action::AddToPlaylist(self.uri.clone()), ActionLabel::AddToPlaylist),
            (key!('b'),Action::AddToQueue(self.uri.clone()), ActionLabel::AddToQueue),
            if self.artists.len() == 1 {
                (key!('A'), Action::GoTo(
                    GoTo::Artist(self.artists.first().unwrap().uri.clone())
                ), ActionLabel::GoToArtist)
            } else {
                (key!('A'), Action::GoTo(
                    GoTo::Artists(
                        self.artists.iter().map(|a| (a.uri.clone(), a.name.clone())).collect::<Vec<_>>()
                    )
                ), ActionLabel::SelectArtist)
            }
        ];

        if context {
            if self.album.total_tracks > 1 {
                actions.push((
                    key!('c'),
                    Action::PlayContext(Play::album(self.album.uri.clone(), None, 0)),
                    ActionLabel::PlayAlbum
                ));
            }
            actions.push((
                key!('C'),
                Action::GoTo(GoTo::Album(self.album.uri.clone())),
                ActionLabel::GoToAlbum
            ))
        }

        actions
    }
}

impl IntoActions for &SimplifiedTrack {
    fn into_ui_actions(self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let actions = vec![
            (key!('p'), Action::AddToPlaylist(self.uri.clone()), ActionLabel::AddToPlaylist),
            (key!('b'), Action::AddToQueue(self.uri.clone()), ActionLabel::AddToQueue),
            if self.artists.len() == 1 {
                (key!('A'), Action::GoTo(
                    GoTo::Artist(self.artists.first().unwrap().uri.clone())
                ), ActionLabel::GoToArtist)
            } else {
                (key!('A'), Action::GoTo(
                    GoTo::Artists(
                        self.artists.iter().map(|a| (a.uri.clone(), a.name.clone())).collect::<Vec<_>>()
                    )
                ), ActionLabel::SelectArtist)
            }
        ];

        actions
    }
}

impl IntoActions for &SimplifiedEpisode {
    fn into_ui_actions(self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        vec![
            (key!('p'), Action::AddToPlaylist(self.uri.clone()), ActionLabel::AddToPlaylist),
            (key!('b'), Action::AddToQueue(self.uri.clone()), ActionLabel::AddToQueue),
        ]
    }
}

impl IntoActions for &Episode {
    fn into_ui_actions(self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        let mut actions = vec![
            (key!('p'), Action::AddToPlaylist(self.uri.clone()), ActionLabel::AddToPlaylist),
            (key!('b'), Action::AddToQueue(self.uri.clone()), ActionLabel::AddToQueue),
        ];

        if context {
            if let Some(show) = self.show.as_ref() {
                if show.total_episodes > 1 {
                    actions.push((
                        key!('c'),
                        Action::PlayContext(Play::show(show.uri.clone(), None, 0)),
                        ActionLabel::PlayShow
                    ));
                }
                actions.push((
                    key!('C'),
                    Action::GoTo(GoTo::Show(show.uri.clone())),
                    ActionLabel::GoToShow
                ));
            }
        }

        actions
    }
}

impl IntoActions for &SimplifiedChapter {
    fn into_ui_actions(self, _: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        vec![
            (key!('p'), Action::AddToPlaylist(self.uri.clone()), ActionLabel::AddToPlaylist),
            (key!('b'), Action::AddToQueue(self.uri.clone()), ActionLabel::AddToQueue),
        ]
    }
}

impl IntoActions for &Item {
    fn into_ui_actions(self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        match &self.item {
            tupy::api::response::Item::Track(t) => {
                let mut actions = vec![
                    if !self.saved {
                        (
                            key!('f'),
                            Action::Save(t.uri.clone()),
                            ActionLabel::Save
                        )
                    } else {
                        (
                            key!('r'),
                            Action::Remove(t.uri.clone()),
                            ActionLabel::Remove,
                        )
                    },
                ];
                actions.extend(t.into_ui_actions(context));
                actions
            },
            tupy::api::response::Item::Episode(e) => {
                let mut actions = vec![
                    if !self.saved {
                        (
                            key!('f'),
                            Action::Save(e.uri.clone()),
                            ActionLabel::Save
                        )
                    } else {
                        (
                            key!('r'),
                            Action::Remove(e.uri.clone()),
                            ActionLabel::Remove,
                        )
                    },
                ];
                actions.extend(e.into_ui_actions(context));
                actions
            }
        }
    }
}

impl IntoActions for &PlaybackState {
    fn into_ui_actions(self, context: bool) -> Vec<(KeyEvent, Action, &'static str)> {
        if let Some(pb) = self.playback.as_ref() {
            match &pb.item {
                PlaybackItem::Track(t) => {
                    let mut actions = vec![
                        // TODO: Wrap the playback fetching on if it is saved. If it has the
                        // functionality then add the action to save/remove it from saved items
                        if !pb.saved {
                            (
                                key!('f'),
                                Action::Save(t.uri.clone()),
                                ActionLabel::Save
                            )
                        } else {
                            (
                                key!('r'),
                                Action::Remove(t.uri.clone()),
                                ActionLabel::Remove,
                            )
                        },
                        (
                            key!('p'),
                            Action::AddToPlaylist(t.uri.clone()),
                            ActionLabel::AddToPlaylist
                        ),
                    ];

                    if context {
                        if t.album.total_tracks > 1 {
                            actions.push((
                                key!('c'),
                                Action::PlayContext(Play::album(t.album.uri.clone(), None, 0)),
                                ActionLabel::PlayAlbum
                            ));
                        }
                        actions.push((
                            key!('A'),
                            Action::GoTo(GoTo::Album(t.album.uri.clone())),
                            ActionLabel::GoToAlbum,
                        ));
                        match pb.context.as_ref() {
                            Some(Context{ uri, .. }) => {
                                actions.push((
                                    key!('C'), Action::GoTo(GoTo::try_from(uri.clone()).unwrap()), ActionLabel::GoToContext
                                ))
                            },
                            None => {}
                        }
                    }

                    actions
                }
                PlaybackItem::Episode(e) => {
                    let mut actions = vec![
                        if !pb.saved {
                            (
                                key!('f'),
                                Action::Save(e.uri.clone()),
                                ActionLabel::Save
                            )
                        } else {
                            (
                                key!('r'),
                                Action::Remove(e.uri.clone()),
                                ActionLabel::Remove,
                            )
                        },
                        (
                            key!('p'),
                            Action::AddToPlaylist(e.uri.clone()),
                            ActionLabel::AddToPlaylist
                        ),
                    ];
                    if context {
                        if let Some(show) = e.show.as_ref() {
                            if show.total_episodes > 1 {
                                actions.push((
                                    key!('c'),
                                    Action::PlayContext(Play::show(show.uri.clone(), None, 0)),
                                    ActionLabel::PlayShow
                                ));
                            }
                            actions.push((
                                key!('C'),
                                Action::GoTo(GoTo::Show(show.uri.clone())),
                                ActionLabel::GoToShow
                            ));
                        }
                    }
                    actions
                },
                _ => Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}

impl Widget for State {
    fn render(mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(4)])
            .split(area);

        //let mut dimmed = if let Viewport::Modal(_) = &self.viewport {
        //    Style::default().dim()
        //} else {
        //    Style::default()
        //};
        let mut dimmed = Style::default();

        match &mut self.window {
            Window::Queue => {
                let qstate = &mut *self.window_state.queue.lock().unwrap();
                StatefulWidget::render(qstate, layout[0], buf, &mut dimmed);
            }
            Window::Library => {
                Widget::render(&*self.window_state.library.lock().unwrap(), layout[0], buf);
            },
            Window::Landing => {
                Widget::render(&mut *self.window_state.landing.lock().unwrap(), layout[0], buf);
            }
        }

        // Viewport State Rendering
        if let Viewport::Modal(modal) = &mut self.viewport {
            match modal {
                Modal::Devices => {
                    let devices = &mut *self.modal_state.devices.lock().unwrap();
                    Widget::render(devices, layout[0], buf);
                }
                Modal::GoTo => {
                    let goto = &self.modal_state.go_to.lock().unwrap();
                    Widget::render(UiGoto(&goto.mappings), layout[0], buf);
                },
                Modal::Action => {
                    let actions = &*self.modal_state.actions.lock().unwrap();
                    Widget::render(actions, layout[0], buf);
                }
                Modal::AddToPlaylist => {
                    let add_to_playlist = &mut *self.modal_state.add_to_playlist.lock().unwrap();
                    // TODO:
                    // - Fetch playlists
                    // - Render playlists in modal
                    // - Use loading state
                    if let Some(add_to_playlist) = add_to_playlist.as_mut() {
                        Widget::render(add_to_playlist, layout[0], buf);
                    }
                }
                Modal::Artists => {
                    let artists = &mut *self.modal_state.artists.lock().unwrap();
                    Widget::render(artists, layout[0], buf);
                }
            }
        }

        Widget::render(&*self.playback.lock().unwrap(), layout[1], buf);
    }
}

pub struct Colors {
    pub artists: Style,
    pub track: Style,
    pub episode: Style,
    pub duration: Style,
    pub context: Style,
    pub like: Style,
    pub finished: Style,
    pub chapter_number: Style,
    pub highlight: Style,
}

lazy_static! {
    pub static ref COLORS: Colors = Colors {
        artists: Style::default().gray(),
        track: Style::default().cyan(),
        episode: Style::default().light_green(),
        duration: Style::default().bold(),
        context: Style::default().white().bold(),
        like: Style::default().fg(Color::Red),
        finished: Style::default().green(),
        chapter_number: Style::default().dim().gray(),
        highlight: Style::default().fg(Color::Yellow),
    };
}

pub struct PaginationProgress {
    pub current: usize,
    pub total: usize,
}

impl Widget for PaginationProgress {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if self.total <= 1 {
            return;
        }

        let mut cells = vec![];
        for _ in 1..(self.current) {
            cells.push(Span::from(DOT).dim().gray());
        }
        cells.push(Span::from(DOT).white().bold());
        for _ in 0..(self.total.saturating_sub(self.current)) {
            cells.push(Span::from(DOT).dim().gray());
        }
    
        let vert = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1)
        ]).split(area)[1];
        Line::from(cells)
            .centered()
            .render(vert, buf);
    }
}
