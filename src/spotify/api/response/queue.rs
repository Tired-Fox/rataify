use serde::{Deserialize, Serialize};

use super::Item;

#[derive(Debug, Clone, Deserialize)]
pub struct Queue {
    pub currently_playing: Option<Item>,
    pub queue: Vec<Item>,
}
