use std::collections::HashMap;

use ratatui::{layout::Constraint, widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, StatefulWidget, Table, TableState, Widget}};

use crate::{action::Action, app::ContextSender, input::Key, state::InnerState, Error, key};

use super::{modal_layout, ModalPosition};

#[derive(Default, Debug, Clone)]
pub struct ActionsState {
    pub mappings: HashMap<Key, Action>,
}

impl ActionsState {
    pub fn set_actions(&mut self, actions: HashMap<Key, Action>) {
        self.mappings = actions;
    }

    pub fn handle(&mut self, action: Action, sender: ContextSender) -> Result<(), Error> {
        match action {
            Action::Select => {
                if let Some(action) = self.mappings.get(&key!(Enter)) {
                    sender.send_action(action.clone())?;
                    sender.send_action(Action::Close)?;
                }
            },
            Action::Key(key) => {
                if let Some(action) = self.mappings.get(&Key::from(key)) {
                    sender.send_action(action.clone())?;
                    sender.send_action(Action::Close)?;
                }
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct Actions;
impl StatefulWidget for Actions {
    type State = InnerState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let mappings = {
            let actions = state.actions.lock().unwrap();
            actions.mappings.clone()
        };

        let mut lk = 0;
        let mut lv = 0;
        let lines = mappings.iter().map(|(k, a)| {
            let key = k.to_string();
            let value = a.label().unwrap();

            if key.len() > lk {
                lk = key.len();
            }

            if value.len() > lv {
                lv = value.len();
            }

            Row::new(vec![
                Cell::new(key),
                Cell::new(value),
            ])
        }).collect::<Vec<_>>();


        let layout = modal_layout(area, Constraint::Length((lv + lk + 6) as u16), Constraint::Length(lines.len() as u16 + 2), ModalPosition::BottomRight);

        Clear.render(layout, buf);

        let block = Block::new()
            .title("Actions")
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        (&block).render(layout, buf);

        let mut pos = TableState::default().with_selected(None);
        let list = Table::new(lines, [
            Constraint::Length(lk as u16 + 1),
            Constraint::Length(lv as u16),
        ]);
        StatefulWidget::render(list, block.inner(layout), buf, &mut pos);
    }
}
