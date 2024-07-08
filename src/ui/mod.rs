use lazy_static::lazy_static;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, symbols::{border, DOT}, text::{Line, Span}, widgets::{block::{Position, Title}, Block, Cell, Row, StatefulWidget, Widget}, style::{Color, Style, Stylize}};
use tupy::{api::{request::Play, response::{Episode, PlaybackItem, Track, SimplifiedTrack, SimplifiedEpisode, SimplifiedChapter}}, Duration};

pub mod modal;
pub mod window;
pub mod playback;
pub mod action;

pub use action::{Action, GoTo};

pub use playback::NoPlayback;

use crate::state::{playback::{Item, PlaybackState}, Modal, State, Viewport, Window};

use self::modal::{actions::ModalActions, add_to_playlist::AddToPlaylist, goto::UiGoto};

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
            Cell::from(Line::from(vec![
                Span::from(episode.name.clone()).style(COLORS.episode),
                Span::from(" ✓").style(COLORS.finished)
            ]))
        } else {
            Cell::from(episode.name.clone()).style(COLORS.episode)
        }
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
            Cell::from(Line::from(vec![
                Span::from(episode.name.clone()).style(COLORS.episode),
                Span::from(" ✓").style(COLORS.finished)
            ]))
        } else {
            Cell::from(episode.name.clone()).style(COLORS.episode)
        }
    ];

    if let Some(show) = episode.show.as_ref() {
        cells.extend([
            Cell::from(show.name.clone()).style(COLORS.context),
        ]);
    } else {
        cells.extend([Cell::default()]);
    }

    Row::new(cells)
}

pub trait IntoActions {
    fn into_ui_actions(self, context: bool) -> Vec<Action>;
}

impl IntoActions for &Track {
    fn into_ui_actions(self, context: bool) -> Vec<Action> {
        let mut actions = vec![
            Action::Play(self.uri.clone()),
            Action::AddToPlaylist(self.uri.clone()),
            Action::AddToQueue(self.uri.clone()),
        ];

        if context {
            if self.album.total_tracks > 1 {
                actions.push(Action::PlayContext(Play::album(self.album.uri.clone(), None, 0)));
            }
            actions.push(Action::GoTo(GoTo::Album(self.album.uri.clone())))
        }

        actions
    }
}

impl IntoActions for &SimplifiedTrack {
    fn into_ui_actions(self, _: bool) -> Vec<Action> {
        let mut actions = vec![
            Action::Play(self.uri.clone()),
            Action::AddToPlaylist(self.uri.clone()),
            Action::AddToQueue(self.uri.clone()),
        ];

        actions
    }
}

impl IntoActions for &SimplifiedEpisode {
    fn into_ui_actions(self, _: bool) -> Vec<Action> {
        vec![
            Action::Play(self.uri.clone()),
            Action::AddToPlaylist(self.uri.clone()),
            Action::AddToQueue(self.uri.clone()),
        ]
    }
}

impl IntoActions for &Episode {
    fn into_ui_actions(self, context: bool) -> Vec<Action> {
        let mut actions = vec![
            Action::Play(self.uri.clone()),
            Action::AddToPlaylist(self.uri.clone()),
            Action::AddToQueue(self.uri.clone()),
        ];

        if context {
            if let Some(show) = self.show.as_ref() {
                if show.total_episodes > 1 {
                    actions.push(Action::PlayContext(Play::show(show.uri.clone(), None, 0)));
                }
                actions.push(Action::GoTo(GoTo::Show(show.uri.clone())));
            }
        }

        actions
    }
}

impl IntoActions for &SimplifiedChapter {
    fn into_ui_actions(self, _: bool) -> Vec<Action> {
        vec![
            Action::Play(self.uri.clone()),
            Action::AddToPlaylist(self.uri.clone()),
            Action::AddToQueue(self.uri.clone()),
        ]
    }
}

impl IntoActions for &Item {
    fn into_ui_actions(self, context: bool) -> Vec<Action> {
        match &self.item {
            tupy::api::response::Item::Track(t) => {
                let mut actions = vec![if !self.saved { Action::Save(t.uri.clone()) } else { Action::Remove(t.uri.clone()) }];
                actions.extend(t.into_ui_actions(context));
                actions
            },
            tupy::api::response::Item::Episode(e) => {
                let mut actions = vec![if !self.saved { Action::Save(e.uri.clone()) } else { Action::Remove(e.uri.clone()) }];
                actions.extend(e.into_ui_actions(context));
                actions
            }
        }
    }
}

impl IntoActions for &PlaybackState {
    fn into_ui_actions(self, context: bool) -> Vec<Action> {
        if let Some(pb) = self.playback.as_ref() {
            match &pb.item {
                PlaybackItem::Track(t) => {
                    let mut actions = vec![
                        // TODO: Wrap the playback fetching on if it is saved. If it has the
                        // functionality then add the action to save/remove it from saved items
                        
                        if !pb.saved { Action::Save(t.uri.clone()) } else { Action::Remove(t.uri.clone()) },
                        Action::AddToPlaylist(t.uri.clone()),
                    ];

                    if context {
                        if t.album.total_tracks > 1 {
                            actions.push(Action::PlayContext(Play::album(t.album.uri.clone(), None, 0)));
                        }
                        actions.push(Action::GoTo(GoTo::Album(t.album.uri.clone())));
                    }

                    actions
                }
                PlaybackItem::Episode(e) => {
                    let mut actions = vec![
                        if !pb.saved { Action::Save(e.uri.clone()) } else { Action::Remove(e.uri.clone()) },
                        Action::AddToPlaylist(e.uri.clone()),
                    ];
                    if context {
                        if let Some(show) = e.show.as_ref() {
                            if show.total_episodes > 1 {
                                actions.push(Action::PlayContext(Play::show(show.uri.clone(), None, 0)));
                            }
                            actions.push(Action::GoTo(GoTo::Show(show.uri.clone())));
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
                    let actions = &self.modal_state.actions.lock().unwrap();
                    Widget::render(ModalActions(actions), layout[0], buf);
                }
                Modal::AddToPlaylist => {
                    let add_to_playlist = &self.modal_state.add_to_playlist.lock().unwrap();
                    // TODO:
                    // - Fetch playlists
                    // - Render playlists in modal
                    // - Use loading state
                    Widget::render(AddToPlaylist(add_to_playlist.as_ref()), layout[0], buf);
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
    };
}

pub struct PaginationProgress {
    pub current: usize,
    pub total: usize,
}

impl Widget for PaginationProgress {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {

        let mut cells = Vec::new();
        for _ in 0..(self.current.saturating_sub(1)) {
            cells.push(Span::from(DOT).dim().gray());
        }
        cells.push(Span::from(DOT).white().bold());
        for _ in 0..(self.total - self.current) {
            cells.push(Span::from(DOT).dim().gray());
        }
    
        Line::from(cells)
            .centered()
            .render(area, buf);
    }
}
