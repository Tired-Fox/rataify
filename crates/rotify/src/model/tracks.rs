use serde::Deserialize;

use crate::model::paginate::{Paginate, parse_pagination};
use crate::model::playback::Track;

#[derive(Debug, Deserialize)]
pub struct PagedItem {
    pub track: Track,
    pub added_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PagedTracks {
    pub href: String,
    pub limit: usize,
    pub offset: usize,
    /// Link to get next page of tracks
    #[serde(deserialize_with = "parse_pagination")]
    pub next: Option<Paginate>,
    /// Link to get previous page of tracks
    #[serde(deserialize_with = "parse_pagination")]
    pub previous: Option<Paginate>,
    pub total: Option<u64>,
    pub items: Vec<PagedItem>,
}

impl PagedTracks {
    /// Index of the last track in the items.
    ///
    /// This value is based on the offset and the page size of the items.
    pub fn last_index(&self) -> usize {
        self.offset + self.limit
    }
}

#[derive(Debug, Deserialize)]
pub struct Recommendations {
    pub seeds: Vec<RecommendationSeed>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecommendationSeed {
    #[serde(rename = "afterFilteringSize")]
    pub after_filter_size: u32,
    #[serde(rename = "afterRelinkingSize")]
    pub after_relinking_size: u32,
    pub href: String,
    pub id: String,
    #[serde(rename = "initialPoolSize")]
    pub initial_pool_size: u32,
    #[serde(rename = "type")]
    pub _type: String,
}
