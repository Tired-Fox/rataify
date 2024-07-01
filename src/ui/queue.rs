use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{block::{Position, Title}, Block, List, ListDirection, Padding, Paragraph, StatefulWidget, Widget},
};
use tupy::api::response::Item;

use crate::state::{Loading, QueueState};

use super::{format_episode, format_track};

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
                    .padding(Padding::uniform(1))
                    .title(title)
                    .border_set(border::ROUNDED);

                // TODO: Handle rendering in scrollable area ???
                let list = q
                    .items
                    .iter()
                    .map(|item| match &item.item {
                        // TODO: Format each line for specific item type
                        Item::Track(t) => format_track(t, item.saved),
                        Item::Episode(e) => format_episode(e),
                    })
                    .collect::<List>()
                    .block(block)
                    .style(*state)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    )
                    .direction(ListDirection::TopToBottom);

                StatefulWidget::render(list, area, buf, &mut self.state);
            }
        }
    }
}
