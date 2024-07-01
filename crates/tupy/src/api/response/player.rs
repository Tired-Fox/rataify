use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Duration, Local};
use serde::Deserialize;

use crate::api::IntoSpotifyParam;

use super::{deserialize_datetime, deserialize_duration_opt, deserialize_timestamp, Cursors, Episode, ExternalUrls, Item, Paged, Track};

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Computer,
    Smartphone,
    Speaker,
    Other(String)
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(s) => write!(f, "{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

impl<'de> Deserialize<'de> for DeviceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "computer" => DeviceType::Computer,
            "smartphone" => DeviceType::Smartphone,
            "speaker" => DeviceType::Speaker,
            _ => DeviceType::Other(s),
        })
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Device {
    /// The device ID. This ID is unique and persistent to some extent. However, this is not guaranteed and any cached device_id should periodically be cleared out and refetched as necessary.
    pub id: String,
    /// If this device is the currently active device.
    pub is_active: bool,
    /// If this device is currently in a private session.
    pub is_private_session: bool,
    /// Whether controlling this device is restricted. At present if this is "true" then no Web API commands will be accepted by this device.
    pub is_restricted: bool,
    /// A human-readable name for the device. Some devices have a name that the user can configure (e.g. "Loudest speaker") and some devices have a generic name associated with the manufacturer or device model.
    pub name: String,
    /// Device type, such as "computer", "smartphone" or "speaker".
    #[serde(rename = "type")]
    pub device_type: DeviceType,
    /// The current volume in percent.
    pub volume_percent: u8,
    /// If this device can be used to set the volume.
    pub supports_volume: bool,
}

#[derive(Default, Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    /// Repeat off.
    #[default]
    Off,
    /// Repeat the current track
    Track,
    /// Repeat the current context
    Context,
}

impl IntoSpotifyParam for Repeat {
    fn into_spotify_param(self) -> Option<String> {
        Some(match self {
            Repeat::Off => "off".to_string(),
            Repeat::Track => "track".to_string(),
            Repeat::Context => "context".to_string(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Context {
    /// The object type, e.g. "artist", "playlist", "album", "show".
    #[serde(rename = "type")]
    pub context_type: String,
    /// A link to the Web API endpoint providing full details of the track.
    pub href: String,
    /// External URLs for this context.
    pub external_urls: Option<ExternalUrls>,
    /// The URI of the context.
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackAction {
    /// Interrupting playback. Optional field.
    InterruptingPlayback,
    /// Pausing. Optional field.
    Pausing,
    /// Resuming. Optional field.
    Resuming,
    /// Seeking playback location. Optional field.
    Seeking,
    /// Skipping to the next context. Optional field.
    SkippingNext,
    //// Skipping to the previous context. Optional field.
    SkippingPrev,
    /// Toggling repeat context flag. Optional field.
    TogglingRepeatContext,
    /// Toggling shuffle flag. Optional field.
    TogglingShuffle,
    /// Toggling repeat track flag. Optional field.
    TogglingRepeatTrack,
    /// Transfering playback between devices. Optional field.
    TransferringPlayback,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackActionScope {
    Disallows,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag="currently_playing_type", content="item", rename_all = "snake_case")]
pub enum PlaybackItem {
    Track(Box<Track>),
    Episode(Box<Episode>),
    Ad,
    Unkown,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Playback {
    /// The device that is currently active.
    pub device: Option<Device>,
    /// off, track, context
    #[serde(default, rename = "repeat_state")]
    pub repeat: Repeat,
    /// If shuffle is on or off.
    #[serde(default, rename = "shuffle_state")]
    pub shuffle: bool,
    /// A Context Object.
    pub context: Option<Context>,
    /// Unix Millisecond Timestamp when playback state was last changed (play, pause, skip, scrub, new song, etc.).
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: DateTime<Local>,
    // Progress into the currently playing track or episode. Can be null.
    #[serde(rename = "progress_ms", deserialize_with = "deserialize_duration_opt")]
    pub progress: Option<Duration>,
    /// If something is currently playing, return true.
    pub is_playing: bool,
    /// The currently playing track or episode.
    #[serde(flatten)]
    pub item: PlaybackItem,
    /// Allows to update the user interface based on which playback actions are available within the current context.
    pub actions: HashMap<PlaybackActionScope, HashMap<PlaybackAction, bool>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PlayHistory {
    ///The track the user listened to.
    pub track: Track,
    /// The date and time the track was played.
    #[serde(deserialize_with = "deserialize_datetime")]
    pub played_at: DateTime<Local>,
    /// The context the track was played from.
    pub context: Context,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecentlyPlayed {
    /// A link to the Web API endpoint returning the full result of the request.
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// The number of items returned in the response (as set in the query or by default).
    #[serde(default)]
    pub total: usize,
    /// URL to the next page of items. ( null if none)
    pub next: Option<String>,
    /// The cursors used to find the next set of items.
    pub cursors: Option<Cursors>,
    /// The items returned in the response.
    pub items: Vec<PlayHistory>,
}

impl Paged for RecentlyPlayed {
    type Item = PlayHistory;

    fn next(&self) -> Option<&str> {
        self.next.as_deref()
    }

    fn prev(&self) -> Option<&str> {
        None
    }

    fn page(&self) -> usize {
       0 
    }

    fn limit(&self) -> usize {
        self.limit
    }

    fn total(&self) -> usize {
       self.total
    }

    fn items(&self) -> &Vec<Self::Item> {
        &self.items
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Queue {
    /// The currently playing track or episode. Can be null.
    pub currently_playing: Item,
    /// The tracks or episodes in the queue. Can be empty.
    pub queue: Vec<Item>,
}
