use serde::Deserialize;
use crate::model::playback::{Context, Item, Track};

#[derive(Debug, Deserialize)]
pub struct RecentlyPlayedCursors {
    pub after: String,
    pub before: String,
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
    pub limit: Option<u8>,
    pub next: Option<String>,
    pub cursors: RecentlyPlayedCursors,
    pub total: Option<u64>,
    pub items: Vec<RecentlyPlayedItems>,
}

#[derive(Debug, Deserialize)]
pub struct Queue {
    currently_playing: Item,
    pub queue: Vec<Item>,
}