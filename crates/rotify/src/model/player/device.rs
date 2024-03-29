use std::hash::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Devices {
    pub devices: Vec<Device>,
}

/// Spotify's representation of a device that can be streamed to
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_percent: Option<u32>,
    /// If this device can be used to set the volume
    pub supports_volume: bool,
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Device {}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.r#type.hash(state);
    }
}
