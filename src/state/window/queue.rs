use crossterm::event::KeyEvent;
use ratatui::widgets::TableState;
use tupy::api::response;

use crate::{state::{IterCollection, Loading, playback::Item}, ui::{Action, IntoActions}};

#[derive(Debug, Clone, PartialEq)]
pub struct Queue {
    pub items: Vec<Item>,
}

impl From<(response::Queue, Vec<bool>, Vec<bool>)> for Queue {
    fn from(q: (response::Queue, Vec<bool>, Vec<bool>)) -> Self {
        let mut saved_tracks = q.1.into_iter();
        let mut saved_episodes = q.2.into_iter();
        Self {
            items: q.0.queue.into_iter().map(|i| match &i {
                response::Item::Track(_) => Item::new(i, saved_tracks.next().unwrap_or(false)),
                response::Item::Episode(_) => Item::new(i, saved_episodes.next().unwrap_or(false)),
            }).collect(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QueueState {
    pub state: TableState,
    pub queue: Loading<Queue>,
}

impl QueueState {
    pub fn next(&mut self) {
        if let Loading::Some(ref mut q) = self.queue {
            self.state.next_in_list(q.items.len());
        }
    }
    
    pub fn prev(&mut self) {
        if let Loading::Some(ref mut q) = self.queue {
            self.state.prev_in_list(q.items.len());
        }
    }

    pub fn select(&self) -> Option<Vec<(KeyEvent, Action)>> {
        if let Loading::Some(ref q) = self.queue {
            q.items.get(self.state.selected().unwrap_or(0)).map(|i| i.into_ui_actions(true))
        } else {
            None
        }
    }
}

