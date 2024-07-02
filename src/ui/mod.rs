use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::Stylize, text::Line, widgets::{Cell, Row}};
use tupy::{api::response::{Episode, Track}, Duration};

pub mod modal;
pub mod playback;
pub mod queue;
pub mod action;

pub use playback::NoPlayback;

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
fn format_episode<'l>(episode: &Episode) -> Row<'l> {
    let mut cells = vec![
        Cell::from(if episode.resume_point.fully_played { "✓" } else { "" }),
        Cell::from(Line::from(format_duration(episode.duration)).right_aligned()),
        Cell::from(episode.name.clone()).green(),
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
