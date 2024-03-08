use std::collections::HashMap;
use serde::Deserialize;

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