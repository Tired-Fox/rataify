use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Constraint, widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, StatefulWidget, Table, TableState, Widget}};

use crate::{action::{Action, Open}, app::ContextSender, state::InnerState, Error};

use super::{modal_layout, ModalPosition};

pub struct GoTo;
impl GoTo {
    pub fn handle_action(action: Action, context: ContextSender) -> Result<(), Error> {
        match action {
            Action::Key(key) => match key {
                KeyEvent { code: KeyCode::Char('L'), .. } => context.send_action(Action::Open(Open::Library))?,
                _ => {}
            }
            _ => {}
        }
        Ok(())
    }
}

impl StatefulWidget for GoTo {
    type State = InnerState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, _state: &mut Self::State) {
        let layout = modal_layout(area, Constraint::Length(15), Constraint::Length(3), ModalPosition::BottomRight);

        Clear.render(layout, buf);

        let block = Block::new()
            .title("Goto")
            .padding(Padding::horizontal(2))
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        Widget::render(&block, layout, buf);

        let mut pos = TableState::default();
        let table = Table::new([
            Row::new(vec![
                Cell::from("L"),
                Cell::from("Library"),
            ])
        ],
        [
            Constraint::Length(1),
            Constraint::Fill(7),
        ]);
        StatefulWidget::render(table, block.inner(layout), buf, &mut pos);
    }
}
