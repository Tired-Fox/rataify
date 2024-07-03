use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::Stylize, text::{Line, Span}, widgets::{Cell, Row}};
use tupy::{api::{request::Play, response::{Episode, PlaybackItem, Track}}, Duration};

pub mod modal;
pub mod playback;
pub mod queue;
pub mod action;

pub use playback::NoPlayback;

use crate::state::{Item, PlaybackState};

use self::action::{GoTo, UiAction};

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
fn format_track<'l>(track: &Track, saved: bool) -> Row<'l> {
    Row::new(vec![
        Cell::from(if saved { "♥" } else { "" }),
        Cell::from(Line::from(format_duration(track.duration)).right_aligned()),
        Cell::from(track.name.clone()).cyan(),
        //Cell::from(track.album.name.clone()).italic().yellow(),
        Cell::from(
            track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ),
    ])
}

/// Icon | Duration | Name | By | Context
fn format_episode<'l>(episode: &Episode, saved: bool) -> Row<'l> {
    let mut cells = vec![
        Cell::from(if saved { "♥" } else { "" }),
        Cell::from(Line::from(format_duration(episode.duration)).right_aligned()),
        if episode.resume_point.fully_played {
            Cell::from(Line::from(vec![
                Span::from(episode.name.clone()),
                Span::from(" ✓").green()
            ])).green()
        } else {
            Cell::from(episode.name.clone()).green()
        }
    ];

    if let Some(show) = episode.show.as_ref() {
        cells.extend([
            Cell::from(show.name.clone()),
        ]);
    } else {
        cells.extend([Cell::default()]);
    }

    Row::new(cells)
}

pub trait IntoUiActions {
    fn into_ui_actions(self) -> Vec<UiAction>;
}

impl IntoUiActions for Item {
    fn into_ui_actions(self) -> Vec<UiAction> {
        match self.item {
            tupy::api::response::Item::Track(t) => {
                let mut actions = vec![
                    UiAction::Play(t.uri.clone()),
                    if !self.saved { UiAction::Save(t.uri.clone()) } else { UiAction::Remove(t.uri.clone()) },
                    UiAction::AddToPlaylist(t.uri.clone()),
                    UiAction::AddToQueue(t.uri.clone()),
                ];
                if t.album.total_tracks > 1 {
                    actions.push(UiAction::PlayContext(Play::album(t.album.uri.clone(), None, 0)));
                }
                actions
            },
            tupy::api::response::Item::Episode(e) => {
                let mut actions = vec![
                    UiAction::Play(e.uri.clone()),
                    if !self.saved { UiAction::Save(e.uri.clone()) } else { UiAction::Remove(e.uri.clone()) },
                    UiAction::AddToPlaylist(e.uri.clone()),
                    UiAction::AddToQueue(e.uri.clone()),
                ];
                if let Some(show) = e.show.as_ref() {
                    if show.total_episodes > 1 {
                        actions.push(UiAction::PlayContext(Play::show(show.uri.clone(), None, 0)));
                    }
                    actions.push(UiAction::GoTo(GoTo::Show(show.uri.clone())));
                }
                actions
            }
        }
    }
}

impl IntoUiActions for &PlaybackState {
    fn into_ui_actions(self) -> Vec<UiAction> {
        if let Some(pb) = self.playback.as_ref() {
            match &pb.item {
                PlaybackItem::Track(t) => {
                    let mut actions = vec![
                        // TODO: Wrap the playback fetching on if it is saved. If it has the
                        // functionality then add the action to save/remove it from saved items
                        
                        if !self.saved { UiAction::Save(t.uri.clone()) } else { UiAction::Remove(t.uri.clone()) },
                        UiAction::AddToPlaylist(t.uri.clone()),
                    ];

                    if t.album.total_tracks > 1 {
                        actions.push(UiAction::PlayContext(Play::album(t.album.uri.clone(), None, 0)));
                    }

                    actions
                }
                PlaybackItem::Episode(e) => {
                    let mut actions = vec![
                        if !self.saved { UiAction::Save(e.uri.clone()) } else { UiAction::Remove(e.uri.clone()) },
                        UiAction::AddToPlaylist(e.uri.clone()),
                        UiAction::AddToQueue(e.uri.clone()),
                    ];
                    if let Some(show) = e.show.as_ref() {
                        if show.total_episodes > 1 {
                            actions.push(UiAction::PlayContext(Play::show(show.uri.clone(), None, 0)));
                        }
                        actions.push(UiAction::GoTo(GoTo::Show(show.uri.clone())));
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
