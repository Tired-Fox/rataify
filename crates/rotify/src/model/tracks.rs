use serde::Deserialize;

use crate::model::playback::Track;

#[derive(Debug, Deserialize)]
pub struct PagedItem {
    pub track: Track,
    pub added_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PagedTracks {
    pub href: String,
    pub limit: Option<u8>,
    pub offset: Option<u8>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub total: Option<u64>,
    pub items: Vec<PagedItem>,
}
