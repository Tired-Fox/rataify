use serde::Deserialize;
use crate::AsyncIter;
use crate::model::player::{Context, Item, Track};

fn default_limit() -> usize {
    20
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Cursor {
    pub after: Option<String>,
    pub before: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecentlyPlayedItems {
    pub track: Track,
    pub played_at: String,
    pub context: Option<Context>,
}

#[derive(Debug, Deserialize)]
pub struct RecentlyPlayedTracks {
    pub href: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub next: Option<String>,
    pub cursors: Option<Cursor>,
    pub total: Option<usize>,
    pub items: Vec<RecentlyPlayedItems>,
}

#[derive(Debug, Deserialize)]
pub struct Queue {
    currently_playing: Item,
    pub queue: Vec<Item>,
}