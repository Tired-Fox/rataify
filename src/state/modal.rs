use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::widgets::TableState;
use tupy::api::{response::Device, Uri};

use crate::{ui::action::{GoTo, Action}, Shared, Locked};

use super::IterCollection;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DevicesState {
    pub state: TableState,
    pub devices: Vec<Device>,
}

impl DevicesState {
    pub fn next(&mut self) {
        self.state.next_in_list(self.devices.len());
    }
    
    pub fn prev(&mut self) {
        self.state.prev_in_list(self.devices.len());
    }

    pub fn select(&self) -> Device {
        self.devices[self.state.selected().unwrap_or(0)].clone()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GoToState {
    lookup: HashMap<KeyCode, usize>,
    pub mappings: Vec<(KeyCode, GoTo)>,
}

impl GoToState {
    pub fn new(mappings: Vec<(KeyCode, GoTo)>) -> Self {
        Self {
            lookup: mappings.iter().enumerate().map(|(i, (k, _))| (*k, i)).collect(),
            mappings,
        }
    }

    pub fn contains(&self, key: &KeyCode) -> bool {
        self.lookup.contains_key(key)
    }

    pub fn get(&self, key: &KeyCode) -> Option<&GoTo> {
        self.lookup.get(key).map(|i| &self.mappings[*i].1)
    }
}

#[derive(Debug, Clone)]
pub struct ModalState {
    pub devices: Shared<Locked<DevicesState>>,
    pub go_to: Shared<Locked<GoToState>>,
    pub actions: Shared<Locked<Vec<Action>>>,
    pub add_to_playlist: Shared<Locked<Option<Uri>>>,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            devices: Shared::default(),
            go_to: Shared::new(Locked::new(GoToState::new(vec![
                (KeyCode::Char('_'), GoTo::Queue),
                (KeyCode::Char('L'), GoTo::Library),
            ]))),
            actions: Shared::default(),
            add_to_playlist: Shared::default(),
        }
    }
}

