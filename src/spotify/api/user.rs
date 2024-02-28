use std::collections::HashMap;
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use crate::spotify::api::Image;

/// Spotify's representation of a users explicit content settings
#[derive(Debug, Deserialize)]
pub struct ExplicitContent {
    pub filter_enabled: bool,
    pub filter_locked: bool,
}

/// Spotify's representation of an image
#[derive(Debug, Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

/// Spotify's representation of a user profile
#[derive(Debug, Deserialize)]
pub struct User {
    // Private profile information
    country: String,
    email: String,
    explicit_content: ExplicitContent,
    product: String,

    // Public profile information
    pub display_name: String,
    pub external_urls: HashMap<String, String>,
    pub followers: Followers,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub r#type: String,
    pub uri: String,
}

/// Spotify's representation of a device that can be streamed to
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Device {
    pub id: String,
    /// If this is the currently active device
    pub is_active: bool,
    /// If this device is currently in a private session
    pub is_private_session: bool,
    /// Whether controlling this device is restricted. At present if 'true'
    /// then no Web API commands will be accepted by this device
    pub is_restricted: bool,
    pub name: String,
    /// Device type, such as "computer", "smartphone", or "speaker"
    pub r#type: String,
    /// The volume in percent
    ///
    /// Range: `0 - 100`
    pub volume_percent: Option<u32>,
    /// If this device can be used to set the volume
    pub supports_volume: bool,
}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.r#type.hash(state);
    }
}
