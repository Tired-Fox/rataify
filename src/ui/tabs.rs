use lazy_static::lazy_static;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, Paragraph, Row, StatefulWidget, Table, Widget},
};

use super::footer::Cover;

use crate::{
    spotify::response::{Episode, Item, Track},
    state::{MainWindow, State},
};

pub struct Queue;
impl StatefulWidget for Queue {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.queue.queue() {
            Some(queue) => {
                let list = Table::new(
                    queue
                        .iter()
                        .map(|i| {
                            Row::new([match i {
                                Item::Track(Track { name, .. }) => name.clone(),
                                Item::Episode(Episode { name, .. }) => name.clone(),
                            }])
                        })
                        .collect::<Vec<Row>>(),
                    [Constraint::Fill(1)],
                )
                .highlight_style(Style::default().reversed());
                Widget::render(list, area, buf);
            }
            None => {}
        }
    }
}

pub struct Main;

impl StatefulWidget for Main {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Center content horizontally
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(50),
                Constraint::Fill(1),
            ])
            .split(area);

        // Center content vertically, with room for album cover and album name
        let npl = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(15),
                Constraint::Min(1),
            ])
            .split(content_layout[1]);

        let now_playing_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                // Cover
                Constraint::Length(14),
                // Album name
                Constraint::Length(1),
            ])
            .split(npl[1]);

        // 35x14
        let cover_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(35),
                Constraint::Min(1),
            ])
            .split(now_playing_layout[0])[1];

        Cover.render(cover_rect, buf, state);
        Paragraph::new(state.playback.context_name())
            .style(Style::default().bold())
            .alignment(Alignment::Center)
            .render(now_playing_layout[1], buf)
    }
}
