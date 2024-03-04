use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use chrono::{Duration, NaiveDateTime};
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};

use crate::spotify::response::{Device, Followers, Image};

fn ms_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ms: i64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::milliseconds(ms))
}

fn ms_to_duration_optional<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ms: Option<i64> = Deserialize::deserialize(deserializer)?;
    match ms {
        Some(ms) => Ok(Some(Duration::milliseconds(ms))),
        None => Ok(None),
    }
}

fn ms_to_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ms: i64 = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::from_timestamp_millis(ms).ok_or(Error::custom("Invalid timestamp"))
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Repeat {
    Off,
    Track,
    Context,
}

impl Display for Repeat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Repeat::Off => write!(f, "off"),
            Repeat::Track => write!(f, "track"),
            Repeat::Context => write!(f, "context"),
        }
    }
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shuffle {
    On,
    Off,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Context {
    #[serde(rename = "type")]
    pub _type: String,
    pub href: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub external_urls: HashMap<String, String>,
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Album {
    pub album_type: AlbumType,
    pub total_tracks: u32,
    pub available_markets: Vec<String>,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: DatePrecision,
    pub restrictions: Option<HashMap<String, RestrictionReason>>,
    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
    pub artists: Vec<SimplifiedArtist>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Artist {
    pub external_urls: HashMap<String, String>,
    pub followers: Followers,
    pub genres: Vec<String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub popularity: u32,
    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SimplifiedArtist {
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RestrictionReason {
    Explicit,
    Market,
    Product,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Track {
    #[serde(skip)]
    pub liked: bool,
    pub album: Album,
    pub artists: Vec<SimplifiedArtist>,
    pub available_markets: Vec<String>,
    pub disc_number: u32,
    #[serde(rename = "duration_ms", deserialize_with = "ms_to_duration")]
    pub duration: Duration,
    pub explicit: bool,
    pub external_ids: HashMap<String, String>,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: String,
    pub is_playable: Option<bool>,
    // TODO: Change this to be more specific
    pub linked_from: Option<Value>,
    pub restrictions: Option<HashMap<String, RestrictionReason>>,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub track_number: u32,
    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
    pub is_local: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ResumePoint {
    pub fully_played: bool,
    #[serde(rename = "resume_position_ms", deserialize_with = "ms_to_duration")]
    pub resume_position: Duration,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Copyright {
    pub text: String,
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Show {
    pub available_markets: Vec<String>,
    pub copyrights: Vec<Copyright>,
    pub description: String,
    pub html_description: String,
    pub explicit: bool,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub is_externally_hosted: bool,
    pub languages: Vec<String>,
    pub media_type: String,
    pub name: String,
    pub publisher: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub total_episodes: u32,
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Episode {
    #[serde(skip)]
    pub liked: bool,
    pub audio_preview_url: Option<String>,
    pub description: String,
    pub html_description: String,
    #[serde(rename = "duration_ms", deserialize_with = "ms_to_duration")]
    pub duration: Duration,
    pub explicit: bool,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub is_externally_hosted: bool,
    pub is_playable: bool,
    #[deprecated]
    pub language: String,
    pub languages: Vec<String>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: DatePrecision,
    pub resume_point: Option<ResumePoint>,
    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
    pub restrictions: Option<HashMap<String, RestrictionReason>>,
    pub show: Option<Show>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Item {
    Track(Track),
    Episode(Episode),
}

impl Item {
    pub fn id(&self) -> String {
        match self {
            Item::Track(track) => track.id.clone(),
            Item::Episode(episode) => episode.id.clone(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Item::Track(track) => track.name.clone(),
            Item::Episode(episode) => episode.name.clone(),
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            Item::Track(track) => track.duration,
            Item::Episode(episode) => episode.duration,
        }
    }

    pub fn liked(&self) -> bool {
        match self {
            Item::Track(track) => track.liked,
            Item::Episode(episode) => episode.liked,
        }
    }

    pub fn set_liked(&mut self, liked: bool) {
       match self {
           Item::Track(track) => track.liked = liked,
           Item::Episode(episode) => episode.liked = liked,
       }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Playback {
    /// Currently used device
    pub device: Device,
    #[serde(rename = "repeat_state")]
    pub repeat: Repeat,
    #[serde(rename = "shuffle_state")]
    pub shuffle: bool,
    pub context: Option<Context>,
    #[serde(deserialize_with = "ms_to_datetime")]
    pub timestamp: NaiveDateTime,
    #[serde(
        rename = "progress_ms",
        deserialize_with = "ms_to_duration_optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub progress: Option<Duration>,
    #[serde(rename = "is_playing")]
    pub playing: bool,
    currently_playing_type: CurrentlyPlayingType,

    pub item: Option<Item>,
    pub actions: HashMap<String, HashMap<String, bool>>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CurrentlyPlayingType {
    Track,
    Episode,
    #[serde(rename = "ad")]
    Advertisement,
    Unknown,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum AdditionalType {
    Track,
    Episode,
}

