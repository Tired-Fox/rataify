use crossterm::event::KeyEvent;
use ratatui::widgets::TableState;
use tupy::api::response::{self, Item};

use crate::{key, state::{IterCollection, Loading, actions::{Action, action_label}, wrappers::{Saved, GetUri}}};

#[derive(Debug, Clone, PartialEq)]
pub struct Queue {
    pub items: Vec<Saved<Item>>,
}

impl From<(response::Queue, Vec<bool>, Vec<bool>)> for Queue {
    fn from(q: (response::Queue, Vec<bool>, Vec<bool>)) -> Self {
        let mut saved_tracks = q.1.into_iter();
        let mut saved_episodes = q.2.into_iter();
        Self {
            items: q.0.queue.into_iter().map(|i| match &i {
                Item::Track(_) => Saved::new(saved_tracks.next().unwrap_or_default(), i),
                Item::Episode(_) => Saved::new(saved_episodes.next().unwrap_or_default(), i),
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

    pub fn select(&self) -> Option<Vec<(KeyEvent, Action, &'static str)>> {
        if let Loading::Some(ref q) = self.queue {
            q.items.get(self.state.selected().unwrap_or(0)).map(|i| {
                let mut actions = vec![
                    (key!(Enter), Action::Play(i.as_ref().get_uri()), action_label::PLAY)
                ];
                actions.extend(i.into_actions(true, |saved| Ok(())));
                actions
            })
        } else {
            None
        }
    }
}

