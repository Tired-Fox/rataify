use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{block::{Position, Title}, Block, Padding, Paragraph, StatefulWidget, Table, Widget},
};
use tupy::api::response::Item;

use crate::state::{Loading, window::queue::QueueState};

use crate::ui::{format_episode_saved, format_track_saved};

impl StatefulWidget for &mut QueueState {
    type State = Style;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let title = Title::from("[Queue]").alignment(Alignment::Center).position(Position::Bottom);
        match self.queue.as_ref() {
            Loading::Loading => {
                let block = Block::bordered()
                    .padding(Padding::new(0, 0, area.height / 2, 0))
                    .title(title)
                    .border_set(border::ROUNDED);
                Paragraph::new("Loading Queue...")
                    .block(block)
                    .style(*state)
                    .alignment(Alignment::Center)
                    .render(area, buf);
            }
            Loading::None => {
                let block = Block::bordered()
                    .padding(Padding::new(0, 0, area.height / 2, 0))
                    .title(title)
                    .border_set(border::ROUNDED);
                Paragraph::new("<No Queue>")
                    .block(block)
                    .style(*state)
                    .alignment(Alignment::Center)
                    .render(area, buf);
            }
            Loading::Some(q) => {
                let block = Block::bordered()
                    .padding(Padding::symmetric(1, 0))
                    .title(title)
                    .border_set(border::ROUNDED);
                
                //let table = text
                //    .split("\n")
                //    .map(|line: &str| -> Row { line.split_ascii_whitespace().collect() })
                //    .collect::<Table>()
                //    .widths([Constraint::Length(10); 3]);
                // TODO: Handle rendering in scrollable area ???
                
                let max_name = q.items.iter().map(|i| match &i.item {
                    Item::Track(t) => t.name.len(),
                    Item::Episode(e) => e.name.len() + if e.resume_point.fully_played { 2 } else { 0 },
                }).max().unwrap_or(0);

                let table = q
                    .items
                    .iter()
                    .map(|item|  match &item.item {
                        // TODO: Format each line for specific item type
                        Item::Track(t) => format_track_saved(&t, item.saved),
                        Item::Episode(e) => format_episode_saved(&e, item.saved),
                    })
                    .collect::<Table>()
                    .block(block)
                    .style(*state)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    )
                    .widths([
                        Constraint::Length(3),
                        Constraint::Length(8),
                        Constraint::Max(max_name as u16),
                        Constraint::Fill(1),
                        //Constraint::Fill(1),
                    ])
                    .column_spacing(2);

                StatefulWidget::render(table, area, buf, &mut self.state);
            }
        }
    }
}
