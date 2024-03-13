use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::model::paginate::{Paginate, parse_pagination};
use crate::model::player::{Artist, Cursor};

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Tracks,
    Artists,
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", match self {
            Self::Tracks => "tracks",
            Self::Artists => "artists",
        })
    }
}

/// Spotify's representation of an image
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ExplicitContent {
    pub filter_enabled: bool,
    pub filter_locked: bool,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Image {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct UserProfile {
    #[cfg(feature = "user-read-private")]
    pub country: String,
    pub display_name: Option<String>,
    #[cfg(feature = "user-read-email")]
    pub email: String,
    #[cfg(feature = "user-read-private")]
    pub explicit_content: ExplicitContent,
    pub external_urls: HashMap<String, String>,
    pub followers: Followers,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,

    // TODO: See if there are set options, if so add them to enum instead
    #[cfg(feature = "user-read-private")]
    pub product: String,

    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct UserPublicProfile {
    pub display_name: Option<String>,
    pub external_urls: HashMap<String, String>,
    pub followers: Followers,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,

    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
}


#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct TopItems<I: Debug + PartialEq + Clone> {
    pub href: String,
    pub limit: usize,
    #[serde(deserialize_with = "parse_pagination")]
    pub next: Option<Paginate>,
    #[serde(deserialize_with = "parse_pagination")]
    pub previous: Option<Paginate>,
    pub offset: usize,
    pub total: usize,
    pub items: Vec<I>,
}

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
