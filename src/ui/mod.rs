use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tupy::Duration;

pub mod modal;
pub mod playback;
pub mod window;

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
