use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Paragraph, Row, StatefulWidget, Table, Widget},
};

use crate::{
    spotify::response::{Episode, Item, Track},
    state::State,
};

use crate::ui::Cover;
use crate::ui::list_view::{TrackItem, TrackList};

pub struct Queue;
impl StatefulWidget for Queue {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.queue.get_queue() {
            Some(queue) => {
                let tracks = queue
                    .iter()
                    .map(|i| {
                        match i {
                            Item::Track(Track { liked, name, duration, artists, album, .. }) => {
                                TrackItem::new(
                                    name.clone(),
                                    artists.iter().map(|a| a.name.clone()).collect::<Vec<String>>().join(", "),
                                    *duration,
                                   *liked,
                                )
                            },
                            Item::Episode(Episode { liked, name, duration, show, .. }) => {
                                let (artist, album) = match show {
                                    Some(show) => (
                                       show.publisher.clone(),
                                       show.name.clone(),
                                    ),
                                    None => (String::new(), String::new()),
                                };
                                TrackItem::new(
                                    name.clone(),
                                    artist,
                                    *duration,
                                    *liked,
                                )
                            },
                        }
                    })
                    .collect::<Vec<TrackItem>>();

                TrackList::default()
                    .items(tracks)
                    .select(0)
                    .render(area, buf)
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
