use std::fmt::Display;
use chrono::Duration;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Widget};
use ratatui::style::Styled;

#[derive(Debug, Default)]
pub struct TrackItem {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub liked: bool,
}

impl TrackItem {
    pub fn new(name: String, artist: String, duration: Duration, liked: bool) -> Self {
        Self {
            name,
            artist,
            duration,
            album: String::new(),
            liked,
        }
    }
}

#[derive(Debug, Default)]
pub struct TrackList {
    pub items: Vec<TrackItem>,
    pub selected: usize,

    style: Style,
    highlighted_style: Style,
}

impl TrackList {
    pub fn items(mut self, items: Vec<TrackItem>) -> Self {
        self.items = items;
        self
    }

    pub fn select(mut self, selected: usize) -> Self {
        if selected < self.items.len() {
            self.selected = selected;
        }
        self
    }

    pub fn highlighted_style(mut self, style: Style) -> Self {
        self.highlighted_style = style;
        self
    }
}

impl Styled for TrackList {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = Into::<Style>::into(style);
        self
    }
}

impl Widget for TrackList {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        let height = area.height as usize;

        let mut widths: [usize; 2] = [0, 0];
        for item in self.items.iter() {
            if item.name.len() > widths[0] {
                widths[0] = item.name.len();
            }
            if item.artist.len() > widths[1] {
                widths[1] = item.artist.len();
            }
        }

        // TODO: Calculate the scroll: Attempt to keep cursor in the middle of the screen
        for (i, item) in self.items.iter().enumerate() {
            buf.set_string(
                area.left(),
                area.top() + i as u16,
                format!("{}", if item.liked { "♥" } else { " " }),
                self.style
            );
            buf.set_string(
                area.left() + 2,
                area.top() + i as u16,
                format!("{:0>2}:{:0>2}", item.duration.num_seconds() / 60, item.duration.num_seconds() % 60),
                self.style
            );
            buf.set_string(
                area.left() + 9,
                area.top() + i as u16,
                item.name.clone(),
                self.style
            );
            buf.set_string(
                area.left() + (10 + widths[0]) as u16,
                area.top() + i as u16,
                item.artist.clone(),
                self.style
            );
            // buf.set_string(
            //     area.left() + (2 + widths[0] + widths[1]) as u16,
            //     area.top() + i as u16,
            //     item.album.clone(),
            //     self.style
            // );
        }
        // # {duration} {name} {artist} {context}
    }
}