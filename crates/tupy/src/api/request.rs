use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use serde::{Serializer, Serialize, ser::SerializeMap};
use serde_json::json;
use serde_json::Value;

use super::IntoSpotifyParam;

#[macro_export]
macro_rules! spotify_request {
    ($type: ident, $url: literal) => {
        paste::paste! {
            $crate::api::SpotifyRequest::<String>::new(reqwest::Method::[<$type:upper>], format!($url))
        }
    };
    ($type: ident, $url: expr) => {
        paste::paste! {
            $crate::api::SpotifyRequest::<String>::new(reqwest::Method::[<$type:upper>], $url)
        }
    };
    ($type: ident, $url: literal, $($param: expr),*) => {
        paste::paste! {
            $crate::api::SpotifyRequest::<String>::new(reqwest::Method::[<$type:upper>], format!($url, $($param,)*))
        }
    }
}

#[macro_export]
macro_rules! spotify_request_get {
    ($($rest: tt)*) => {
        $crate::spotify_request!(get, $($rest)*)
    }
}

#[macro_export]
macro_rules! spotify_request_post {
    ($($rest: tt)*) => {
        $crate::spotify_request!(post, $($rest)*)
    }
}

#[macro_export]
macro_rules! spotify_request_put {
    ($($rest: tt)*) => {
        $crate::spotify_request!(put, $($rest)*)
    }
}

#[macro_export]
macro_rules! spotify_request_delete {
    ($($rest: tt)*) => {
        $crate::spotify_request!(delete, $($rest)*)
    }
}

use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;

pub use crate::spotify_request_get as get;
pub use crate::spotify_request_post as post;
pub use crate::spotify_request_put as put;
pub use crate::spotify_request_delete as delete;
use crate::Error;

use super::Uri;

pub static SUPPORTED_ITEMS: &str = "track,episode";

pub trait IntoSpotifyId {
    fn into_spotify_id(self) -> String;
}

impl IntoSpotifyId for String {
    fn into_spotify_id(self) -> String {
        self
    }
}

impl IntoSpotifyId for &String {
    fn into_spotify_id(self) -> String {
        self.to_string()
    }
}

impl IntoSpotifyId for &str {
    fn into_spotify_id(self) -> String {
        self.to_string()
    }
}

impl IntoSpotifyId for Uri {
    fn into_spotify_id(self) -> String {
        self.id().to_string()
    }
}

impl IntoSpotifyId for &Uri {
    fn into_spotify_id(self) -> String {
        self.id().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    Short,
    Medium,
    Long
}

impl Display for TimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRange::Short => write!(f, "short_term"),
            TimeRange::Medium => write!(f, "medium_term"),
            TimeRange::Long => write!(f, "long_term"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncludeGroup {
    Album,
    Single,
    AppearsOn,
    Compilation,
}

impl Display for IncludeGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IncludeGroup::Album => write!(f, "album"),
            IncludeGroup::Single => write!(f, "single"),
            IncludeGroup::AppearsOn => write!(f, "appears_on"),
            IncludeGroup::Compilation => write!(f, "compilation"),
        }
    }
}

impl IntoSpotifyParam for IncludeGroup {
    fn into_spotify_param(self) -> Option<String> {
        Some(self.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryTag {
    New,
    Hipster
}

impl Display for QueryTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryTag::New => write!(f, "new"),
            QueryTag::Hipster => write!(f, "hipster"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Text(String),
    Artist(String),
    Album(String),
    Track(String),
    Upc(String),
    Isrc(String),
    Genre(String),
    Tag(QueryTag),
    Year(String),
}

impl Query {
    pub fn text<S: Display>(text: S) -> Self {
        Self::Text(text.to_string())
    }

    pub fn artist<S: Display>(artist: S) -> Self {
        Self::Artist(artist.to_string())
    }

    pub fn album<S: Display>(album: S) -> Self {
        Self::Album(album.to_string())
    }

    pub fn track<S: Display>(track: S) -> Self {
        Self::Track(track.to_string())
    }

    pub fn upc<S: Display>(upc: S) -> Self {
        Self::Upc(upc.to_string())
    }

    pub fn isrc<S: Display>(isrc: S) -> Self {
        Self::Isrc(isrc.to_string())
    }

    pub fn genre<S: Display>(genre: S) -> Self {
        Self::Genre(genre.to_string())
    }

    pub fn tag(tag: QueryTag) -> Self {
        Self::Tag(tag)
    }

    pub fn year<S: Display>(year: S) -> Self {
        Self::Year(year.to_string())
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Text(s) => write!(f, "{}", s),
            Query::Artist(s) => write!(f, "artist:{}", s),
            Query::Album(s) => write!(f, "album:{}", s),
            Query::Track(s) => write!(f, "track:{}", s),
            Query::Upc(s) => write!(f, "upc:{}", s),
            Query::Isrc(s) => write!(f, "isrc:{}", s),
            Query::Genre(s) => write!(f, "genre:{}", s),
            Query::Tag(s) => write!(f, "tag:{}", s),
            Query::Year(s) => write!(f, "year:{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SearchType {
    Album,
    Artist,
    Playlist,
    Track,
    Show,
    Episode,
    Audiobook,
}

impl SearchType {
    #[inline]
    pub fn all() -> &'static [SearchType] {
        &[
            SearchType::Album,
            SearchType::Artist,
            SearchType::Playlist,
            SearchType::Track,
            SearchType::Show,
            SearchType::Episode,
            SearchType::Audiobook,
        ]
    }
}

impl Display for SearchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Album => write!(f, "album"),
            SearchType::Artist => write!(f, "artist"),
            SearchType::Playlist => write!(f, "playlist"),
            SearchType::Track => write!(f, "track"),
            SearchType::Show => write!(f, "show"),
            SearchType::Episode => write!(f, "episode"),
            SearchType::Audiobook => write!(f, "audiobook"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeedId {
    Artist(String),
    Track(String),
    Genre(String),
}

impl SeedId {
    pub fn artist<S: Display>(artist: S) -> Self {
        Self::Artist(artist.to_string())
    }

    pub fn track<S: Display>(track: S) -> Self {
        Self::Track(track.to_string())
    }

    pub fn genre<S: Display>(genre: S) -> Self {
        Self::Genre(genre.to_string())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RecommendationSeed {
    pub seed_ids: Vec<SeedId>,
    pub min_acousticness: Option<f32>,
    pub min_danceability: Option<f32>,
    pub min_duration_ms: Option<u32>,
    pub min_energy: Option<f32>,
    pub min_instrumentalness: Option<f32>,
    pub min_key: Option<u32>,
    pub min_liveness: Option<f32>,
    pub min_loudness: Option<f32>,
    pub min_mode: Option<u32>,
    pub min_popularity: Option<f32>,
    pub min_speechiness: Option<f32>,
    pub min_tempo: Option<f32>,
    pub min_time_signature: Option<u32>,
    pub min_valence: Option<f32>,
    pub max_acousticness: Option<f32>,
    pub max_danceability: Option<f32>,
    pub max_duration_ms: Option<u32>,
    pub max_energy: Option<f32>,
    pub max_instrumentalness: Option<f32>,
    pub max_key: Option<u32>,
    pub max_liveness: Option<f32>,
    pub max_loudness: Option<f32>,
    pub max_mode: Option<u32>,
    pub max_popularity: Option<f32>,
    pub max_speechiness: Option<f32>,
    pub max_tempo: Option<f32>,
    pub max_time_signature: Option<u32>,
    pub max_valence: Option<f32>,
    pub target_acousticness: Option<f32>,
    pub target_danceability: Option<f32>,
    pub target_duration_ms: Option<u32>,
    pub target_energy: Option<f32>,
    pub target_instrumentalness: Option<f32>,
    pub target_key: Option<u32>,
    pub target_liveness: Option<f32>,
    pub target_loudness: Option<f32>,
    pub target_mode: Option<u32>,
    pub target_popularity: Option<f32>,
    pub target_speechiness: Option<f32>,
    pub target_tempo: Option<f32>,
    pub target_time_signature: Option<u32>,
    pub target_valence: Option<f32>,
}

struct Params(HashMap<String, String>);
impl Params {
    pub fn add<K: AsRef<str>, V: Display>(&mut self, key: K, value: V) -> &mut Self {
        self.0.insert(key.as_ref().to_string(), value.to_string());
        self
    }

    pub fn add_opt<K: AsRef<str>, V: Display>(&mut self, key: K, value: Option<V>) -> &mut Self {
        if let Some(v) = value {
            self.add(key, v);
        }
        self
    }
}

impl RecommendationSeed {
    pub fn into_params(&self) -> Result<String, Error> {
        if self.seed_ids.len() > 5 {
            return Err(Error::InvalidArgument("seed", "Seed must contain at most 5 seed IDs".to_string()))
        } else if self.seed_ids.is_empty() {
            return Err(Error::InvalidArgument("seed", "Seed must contain at least one seed ID".to_string()))
        }

        let mut params = Params(HashMap::new());
        let artists = self.seed_ids.iter().filter_map(|id| match id {
            SeedId::Artist(a) => Some(a.as_ref()),
            _ => None,
        }).collect::<Vec<&str>>();
        let tracks = self.seed_ids.iter().filter_map(|id| match id {
            SeedId::Track(t) => Some(t.as_ref()),
            _ => None,
        }).collect::<Vec<&str>>();
        let genres = self.seed_ids.iter().filter_map(|id| match id {
            SeedId::Genre(g) => Some(g.as_ref()),
            _ => None,
        }).collect::<Vec<&str>>();

        if !artists.is_empty() {
            params.add("seed_artists", artists.join(","));
        }

        if !tracks.is_empty() {
            params.add("seed_tracks", tracks.join(","));
        }

        if !genres.is_empty() {
            params.add("seed_genres", genres.join(","));
        }

        params
            .add_opt("min_acousticness", self.min_acousticness)
            .add_opt("min_danceability", self.min_danceability)
            .add_opt("min_duration_ms", self.min_duration_ms)
            .add_opt("min_energy", self.min_energy)
            .add_opt("min_instrumentalness", self.min_instrumentalness)
            .add_opt("min_key", self.min_key)
            .add_opt("min_liveness", self.min_liveness)
            .add_opt("min_loudness", self.min_loudness)
            .add_opt("min_mode", self.min_mode)
            .add_opt("min_popularity", self.min_popularity)
            .add_opt("min_speechiness", self.min_speechiness)
            .add_opt("min_tempo", self.min_tempo)
            .add_opt("min_time_signature", self.min_time_signature)
            .add_opt("min_valence", self.min_valence)
            .add_opt("max_acousticness", self.max_acousticness)
            .add_opt("max_danceability", self.max_danceability)
            .add_opt("max_duration_ms", self.max_duration_ms)
            .add_opt("max_energy", self.max_energy)
            .add_opt("max_instrumentalness", self.max_instrumentalness)
            .add_opt("max_key", self.max_key)
            .add_opt("max_liveness", self.max_liveness)
            .add_opt("max_loudness", self.max_loudness)
            .add_opt("max_mode", self.max_mode)
            .add_opt("max_popularity", self.max_popularity)
            .add_opt("max_speechiness", self.max_speechiness)
            .add_opt("max_tempo", self.max_tempo)
            .add_opt("max_time_signature", self.max_time_signature)
            .add_opt("max_valence", self.max_valence)
            .add_opt("target_acousticness", self.target_acousticness)
            .add_opt("target_danceability", self.target_danceability)
            .add_opt("target_duration_ms", self.target_duration_ms)
            .add_opt("target_energy", self.target_energy)
            .add_opt("target_instrumentalness", self.target_instrumentalness)
            .add_opt("target_key", self.target_key)
            .add_opt("target_liveness", self.target_liveness)
            .add_opt("target_loudness", self.target_loudness)
            .add_opt("target_mode", self.target_mode)
            .add_opt("target_popularity", self.target_popularity)
            .add_opt("target_speechiness", self.target_speechiness)
            .add_opt("target_tempo", self.target_tempo)
            .add_opt("target_time_signature", self.target_time_signature)
            .add_opt("target_valence", self.target_valence);

        Ok(serde_urlencoded::to_string(&params.0)?)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct PlaylistDetails {
    pub name: Option<String>,
    pub public: Option<bool>,
    pub collaborative: Option<bool>,
    pub description: Option<String>,
}

impl PlaylistDetails {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<S: Display>(mut self, name: S) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn public(mut self, public: bool) -> Self {
        self.public = Some(public);
        self
    }

    pub fn collaborative(mut self, collaborative: bool) -> Self {
        self.collaborative = Some(collaborative);
        self
    }

    pub fn description<S: Display>(mut self, description: S) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaylistAction {
    /// Reorders the items in a playlist
    Reorder {
        /// The index of the first item to move
        start: usize,
        /// The number of items to move
        length: usize,
        /// The index to insert the item(s) before
        insert: usize,
    },
    /// Replaces all items in a playlist
    Uris(Vec<Uri>)
}

impl Serialize for PlaylistAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PlaylistAction::Reorder { start, length, insert } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("range_start", start)?;
                map.serialize_entry("range_length", length)?;
                map.serialize_entry("insert_before", insert)?;
                map.end()
            }
            PlaylistAction::Uris(uris) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("uris", &uris.iter().map(|u| u.to_string()).collect::<Vec<String>>())?;
                map.end()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UriWrapper(pub Uri);
impl Serialize for UriWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("uri", &self.0.to_string())?;
        map.end()
    }
}

#[derive(Debug, Clone)]
pub enum Play {
    Artist(String),
    Album {
        id: String,
        offset: Option<usize>,
        position: Duration,
    },
    Playlist {
        id: String,
        offset: Option<usize>,
        position: Duration,
    },
    Queue {
        uris: Vec<Uri>,
        position: Duration,
        offset: Option<usize>,
    },
    Resume
}

pub trait IntoDuration {
    fn into_duration(self) -> Duration;
}

macro_rules! impl_into_duration {
    ($($typ: ty),* , |$self: ident| { $impl: expr }) => {
        $(
            impl IntoDuration for $typ {
                fn into_duration($self) -> Duration {
                    $impl
                }
            }
        )*
    }
}

impl_into_duration!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, |self| { Duration::milliseconds(self as i64) });
impl_into_duration!(f32, f64, |self| {
    Duration::seconds(self as i64) + Duration::milliseconds((self.fract() * 1000.0) as i64)
});

impl IntoDuration for Duration {
    fn into_duration(self) -> Duration {
        self
    }
}

impl Play {
    pub fn artist<I: IntoSpotifyId>(id: I) -> Self {
        Self::Artist(id.into_spotify_id())
    }

    pub fn album<P, I>(id: I, offset: Option<usize>, position: P) -> Self
    where
        P: IntoDuration,
        I: IntoSpotifyId,
    {
        Self::Album {
            id: id.into_spotify_id(),
            offset,
            position: position.into_duration(),
        }
    }

    pub fn playlist<P, I>(id: I, offset: Option<usize>, position: P) -> Self
    where
        P: IntoDuration,
        I: IntoSpotifyId,
    {
        Self::Playlist {
            id: id.into_spotify_id(),
            offset,
            position: position.into_duration(),
        }
    }

    pub fn queue<U, P>(uris: U, position: P, offset: Option<usize>) -> Self
    where
        U: IntoIterator<Item = Uri>,
        P: IntoDuration,
    {
        Self::Queue {
            uris: uris.into_iter().collect(),
            position: position.into_duration(),
            offset,
        }
    }
}

impl Serialize for Play {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        
        match self {
            Play::Artist(id) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("context_uri", &format!("spotify:artist:{id}"))?;
                map.serialize_entry("position", &0)?;
                map.end()
            },
            Play::Album { id, offset, position } => {
                let mut values = HashMap::from([
                    ("context_uri", Value::from(format!("spotify:album:{id}"))),
                    ("position", Value::from(position.num_milliseconds())),
                ]);

                if let Some(offset) = offset {
                    values.insert("offset", json!({
                        "position": offset
                    }));
                }

                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (k, v) in values {
                    map.serialize_entry(k, &v)?;
                }
                map.end()
            },
            Play::Playlist { id, offset, position } => {
                let mut values = HashMap::from([
                    ("context_uri", Value::from(format!("spotify:playlist:{id}"))),
                    ("position", Value::from(position.num_milliseconds())),
                ]);

                if let Some(offset) = offset {
                    values.insert("offset", json!({
                        "position": offset
                    }));
                }

                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (k, v) in values {
                    map.serialize_entry(k, &v)?;
                }
                map.end()
            },
            Play::Queue { uris, position, offset } => {
                let mut values = HashMap::from([
                    ("uris", Value::from(uris.iter().map(|u| u.to_string()).collect::<Vec<String>>())),
                    ("position", Value::from(position.num_milliseconds())),
                ]);

                if let Some(offset) = offset {
                    values.insert("offset", json!({
                        "position": offset
                    }));
                }

                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (k, v) in values {
                    map.serialize_entry(k, &v)?;
                }
                map.end()
            },
            Play::Resume => {
                serializer.serialize_map(Some(0))?.end()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Timestamp {
    Before(DateTime<Local>),
    After(DateTime<Local>),
}

impl Timestamp {
    pub fn name(&self) -> &'static str {
        match self {
            Timestamp::Before(_) => "before",
            Timestamp::After(_) => "after",
        }
    }

    pub fn before_now() -> Self {
        Self::Before(Local::now())
    }

    pub fn after_now() -> Self {
        Self::After(Local::now())
    }
}

impl IntoSpotifyParam for Timestamp {
    fn into_spotify_param(self) -> Option<String> {
        Some(match self {
            Timestamp::Before(dt) => dt.timestamp_millis().to_string(),
            Timestamp::After(dt) => dt.timestamp_millis().to_string(),
        })
    }
}

