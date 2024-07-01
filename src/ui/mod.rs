use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::Stylize, text::{Line, Span}};
use tupy::{api::response::{Episode, Track}, Duration};

pub mod modal;
pub mod playback;
pub mod queue;

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

fn format_track<'l>(track: &Track, saved: bool) -> Line<'l> {
    Line::default().spans(vec![
        Span::from(if saved { "♥  " } else { "   " }),
        Span::from(format_duration(track.duration)),
        Span::from("  "),
        Span::from(track.name.clone()).cyan(),
        Span::from("  "),
        Span::from(track.album.name.clone()).italic().yellow(),
        Span::from("  "),
        Span::from(
            track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ),
    ])
}

//static CHARS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

fn format_episode<'l>(episode: &Episode) -> Line<'l> {
    Line::default().spans(vec![
        Span::from(format!(" {}  ", if episode.resume_point.fully_played { '✓' } else { ' ' })),
        Span::from(format_duration(episode.duration)),
        Span::from("  "),
        Span::from(episode.name.clone()).green(),
        Span::from("    "),
        Span::from(episode.show.as_ref().unwrap().name.clone()),
    ])
}
