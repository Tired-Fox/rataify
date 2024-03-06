use serde::Deserialize;

/// Spotify's representation of an image
#[derive(Debug, Deserialize, PartialEq)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}
