use lazy_static::lazy_static;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, symbols::DOT, text::{Line, Span}, widgets::{Cell, Row, StatefulWidget, Widget}, style::{Color, Style, Stylize}};
use tupy::{api::response::{Episode, Track}, Duration};

pub mod modal;
pub mod window;
pub mod playback;
pub mod components;

pub use playback::NoPlayback;

use crate::state::{Modal, State, Viewport, Window};

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
