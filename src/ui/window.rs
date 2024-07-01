use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, ToLine, ToSpan},
    widgets::{Block, List, ListDirection, ListState, Padding, StatefulWidget, Widget},
};
use tupy::api::response::{Episode, Item, Queue, Track};

use crate::ui::format_duration;

pub struct UiQueue {
    pub queue: Option<Queue>,
    pub state: ListState,
}

impl Widget for UiQueue {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .padding(Padding::uniform(1))
            .title(Line::from("Queue").centered())
            .border_set(border::ROUNDED);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(2),
            ])
            .split(block.inner(area));

        (&block).render(area, buf);

        // TODO: Handle rendering in scrollable area ???
        if let Some(queue) = &self.queue {
            let list = queue
                .queue
                .iter()
                .map(|i| match i {
                    // TODO: Format each line for specific item type
                    Item::Track(t) => format_track(t),
                    Item::Episode(e) => format_episode(e),
                })
                .collect::<List>()
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                )
                .direction(ListDirection::TopToBottom);

            match &queue.currently_playing {
                Item::Track(t) => format_track(t),
                Item::Episode(e) => format_episode(e),
            }
            .render(layout[0], buf);
            StatefulWidget::render(list, layout[2], buf, &mut self.state);
        }
    }
}

fn format_track<'l>(track: &Track) -> Line<'l> {
    Line::default().spans(vec![
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

fn format_episode<'l>(episode: &Episode) -> Line<'l> {
    Line::default().spans(vec![
        Span::from(format_duration(episode.duration)),
        Span::from("  "),
        Span::from(episode.name.clone()).green(),
        Span::from("    "),
        Span::from(episode.show.as_ref().unwrap().name.clone()),
    ])
}
