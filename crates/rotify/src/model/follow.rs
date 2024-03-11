use serde::Deserialize;
use crate::model::playback::Artist;
use crate::model::queue::Cursor;

#[derive(Debug, Deserialize, Clone)]
pub struct FollowedArtists {
    pub href: String,
    pub limit: Option<usize>,
    pub next: Option<String>,
    pub cursor: Option<Cursor>,
    pub total: Option<usize>,
    pub items: Vec<Artist>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Artists {
    pub artists: FollowedArtists,
}