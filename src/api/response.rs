use std::{fmt::{Debug, Display}, str::FromStr};

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use hyper::Method;
use serde::{Deserialize, Deserializer};

use crate::{Error, Pagination};

use super::{flow::AuthFlow, SpotifyRequest, SpotifyResponse};

#[macro_export]
macro_rules! pares {
    ($value: expr) => {
        {
            let jd = &mut serde_json::Deserializer::from_str($value);
            serde_path_to_error::deserialize(jd)
        }
    };
    ($type: ty: $value: expr) => {
        {
            let jd = &mut serde_json::Deserializer::from_str($value);
            serde_path_to_error::deserialize::<_, $type>(jd)
        }
    };
}

pub use crate::pares;

#[derive(Deserialize)]
struct Author {
    pub name: String,
}
fn deserialize_named_objects<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Vec::<Author>::deserialize(deserializer)?;
    Ok(s.iter().map(|a| a.name.clone()).collect())
}

fn deserialize_date_ymd<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}
fn deserialize_date_ym<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m").map_err(serde::de::Error::custom)
}
fn deserialize_date_y<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y").map_err(serde::de::Error::custom)
}

fn deserialize_added_at<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ").map_err(serde::de::Error::custom)?;
    Ok(Local.from_utc_datetime(&naive))
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let ms = i64::deserialize(deserializer)?;
    Ok(chrono::Duration::milliseconds(ms))
}

pub struct Paginated<R, T, F, const N: usize>
where
    F: AuthFlow,
{
    pub(crate) offset: isize,
    pub(crate) page_size: usize,
    pub(crate) flow: F,
    pub(crate) next: Option<String>,
    pub(crate) prev: Option<String>,
    pub(crate) resolve: Box<dyn Fn(T) -> (R, Option<String>, Option<String>)>
}

impl<T, R, F, const N: usize> Paginated<R, T, F, N>
where
    F: AuthFlow,
{
    pub fn new<C>(flow: F, next: Option<String>, prev: Option<String>, resolve: C) -> Self
    where
        C: Fn(T) -> (R, Option<String>, Option<String>) + 'static
    {
        Self {
            offset: -1,
            page_size:  N,
            flow,
            next,
            prev,
            resolve: Box::new(resolve)
        }
    }

    pub fn page(&self) -> usize {
        self.offset.max(0) as usize
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Get the total number of items fetched so far.
    ///
    /// If `prev` is called this value decreases.
    /// This value is equivalent to `page * page_size`.
    pub fn item_count(&self) -> usize {
        self.page_size() * self.page()
    }
}

impl<R, P: Deserialize<'static>, F, const N: usize> Pagination for Paginated<R, P, F, N>
where
    F: AuthFlow,
{
    type Item = R;
    async fn next(&mut self) -> Result<Option<Self::Item>, Error> {
        self.offset += 1;

        let next = match self.next.as_ref() {
            Some(next) => next,
            None => return Ok(None),
        };

        let SpotifyResponse { body, .. } = SpotifyRequest::new(Method::GET, next).send_raw(self.flow.token().await?).await?;
        let eb = body.clone();
        let body = body.into_boxed_str();
        match pares!(P: Box::leak(body)) {
            Ok(item) => {
                let (result, prev, next) = (self.resolve)(item);
                self.next = next;
                self.prev = prev;
                Ok(Some(result))
            },
            Err(e) => {
                eprintln!("{eb:?}");
                Err(Error::custom(e))
            }
        }
    }

    async fn prev(&mut self) -> Result<Option<Self::Item>, Error> {
        if self.offset < 1 {
            return Ok(None);
        }
        self.offset -= 1;

        let prev = match self.prev.as_ref() {
            Some(prev) => prev,
            None => return Ok(None),
        };

        let SpotifyResponse { body, .. } = SpotifyRequest::new(Method::GET, prev).send_raw(self.flow.token().await?).await?;
        let body = body.into_boxed_str();
        match pares!(P: Box::leak(body)) {
            Ok(item) => {
                let (result, prev, next) = (self.resolve)(item);
                self.next = next;
                self.prev = prev;
                Ok(Some(result))
            },
            Err(e) => Err(Error::custom(e))
        }
    }
}

/// Explicit content settings
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExplicitContent {
    /// When true, indicates that explicit content should not be played.
    pub filter_enabled: bool,
    /// When true, indicates that the explicit content setting is locked and can't be changed by the user.
    pub filter_locked: bool,
}

/// External URLs
///
/// Usually just the Spotify URL
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExternalUrls {
    /// The Spotify URL for the object.
    pub spotify: String,
}

/// Followers for a user profile
#[derive(Default, Debug, Clone, Deserialize, PartialEq)]
pub struct Followers {
    /// This will always be set to null, as the Web API does not support it at the moment.
    #[cfg(feature = "future")]
    pub href: Option<String>,
    /// The total number of followers.
    pub total: u32,
}

/// Spofiy Image
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Image {
    /// The source URL of the image
    pub url: String,
    /// The image height in pixels.
    pub height: u32,
    /// The image width in pixels.
    pub width: u32,
}

/// User Profile
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Profile {
    /// The Spotify user ID for the user.
    pub id: String,
    // The name displayed on the user's profile. `None` if not available
    pub display_name: Option<String>,
    /// Information about the followers of the user.
    pub followers: Followers,
    /// A link to the Web API endpoint for this user.
    pub href: String,
    /// The spotify uri for the user
    pub uri: Uri,
    /// Known external URLs for this user.
    pub external_urls: ExternalUrls,
    /// The user's profile image.
    pub images: Vec<Image>,
    /// The user's Spotify subscription level: "premium", "free", etc. (The subscription level "open" can be considered the same as "free".)
    ///
    /// # Scopes
    /// - user-read-private
    pub product: Option<String>,
    /// The country of the user, as set in the user's account profile. An [ISO 3166-1 alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) country code.
    ///
    /// # Scopes
    /// - user-read-private
    pub country: Option<String>,
    /// The user's email address, as entered by the user when creating their account.
    /// _**Important!** This email address is unverified; there is no proof that it actually belongs to the user._
    ///
    /// # Scopes
    /// - user-read-email
    pub email: Option<String>,
    /// The user's explicit content settings.
    ///
    /// # Scopes
    /// - user-read-private
    #[serde(rename = "explicit_content")]
    pub explicit: Option<ExplicitContent>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Resource {
  Artist,
  Album,
  Track,
  Playlist,
  User,
  Show,
  Episode,
  Collection,
  CollectionYourEpisodes,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Resource::Album => "album",
            Resource::Artist => "artist",
            Resource::Track => "track",
            Resource::Playlist => "playlist",
            Resource::User => "user",
            Resource::Show => "show",
            Resource::Episode => "episode",
            Resource::Collection => "collection",
            Resource::CollectionYourEpisodes => "collectionyourepisodes",
        })
    }
}

impl FromStr for Resource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "album" => Ok(Self::Album),
            "artist" => Ok(Self::Artist),
            "track" => Ok(Self::Track),
            "playlist" => Ok(Self::Playlist),
            "user" => Ok(Self::User),
            "show" => Ok(Self::Show),
            "episode" => Ok(Self::Episode),
            "collection" => Ok(Self::Collection),
            "collectionyourepisodes" => Ok(Self::CollectionYourEpisodes),
            _ => Err("Invalid spotify uri".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Uri {
    resource: Resource,
    id: String,
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "spotify:{}:{}", self.resource, self.id)
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.splitn(3, ':').collect::<Vec<_>>();
        let id = parts[2].to_string();
        Ok(Self {
            resource: Resource::from_str(parts[1]).map_err(serde::de::Error::custom)?,
            id
        })
    }
}

impl Uri {
    /// Id of the spotify uri
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Type of the spotify uri
    pub fn resource(&self) -> Resource {
        self.resource
    }
}

/// Album types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
}

impl<'de> Deserialize<'de> for AlbumType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match s.to_ascii_lowercase().as_str() {
            "album" => Ok(Self::Album),
            "single" => Ok(Self::Single),
            "compilation" => Ok(Self::Compilation),
            _ => Err(serde::de::Error::custom("Invalid album type {s:?}: expected one of 'album', 'single' or 'compilation' (case-insensitive)")),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

fn deserialize_restriction_reason<'de, D>(deserializer: D) -> Result<RestrictionReason, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "explicit" => Ok(RestrictionReason::Explicit),
        "market" => Ok(RestrictionReason::Market),
        "product" => Ok(RestrictionReason::Product),
        _ => Ok(RestrictionReason::Other(s)),
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionReason {
    /// The content item is explicit and the user's account is set to not play explicit content.
    Explicit,
    /// The content item is not available in the given market.
    Market,
    /// The content item is not available for the user's subscription type.
    Product,
    Other(String),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Restrictions {
    #[serde(deserialize_with = "deserialize_restriction_reason")]
    reason: RestrictionReason
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SimplifiedTrack {
    /// The artists who performed the track. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
    /// A list of the countries in which the track can be played, identified by their [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
    #[serde(default="Vec::new")]
    pub available_markets: Vec<String>,
    /// The disc number (usually 1 unless the album consists of more than one disc).
    pub disc_number: u8,
    /// The track length in milliseconds.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the track has explicit lyrics ( true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external URLs for this track.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the track.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub id: String,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied. If true, the track is playable in the given market. Otherwise false.
    #[serde(default="bool::default")]
    pub is_playable: bool,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied, and the requested track has been replaced with different track. The track in the linked_from object contains information about the originally requested track.
    pub linked_from: Option<Box<Track>>,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The name of the track.
    pub name: String,
    /// A link to a 30 second preview (MP3 format) of the track.
    pub preview_url: Option<String>,
    /// The number of the track. If an album has several discs, the track number is the number on the specified disc.
    pub track_number: u8,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub uri: Uri,
    /// Whether or not the track is from a local file.
    pub is_local: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SimplifiedArtist {
    /// Known external URLs for this artist.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the artist.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    pub id: String,
    /// The name of the artist.
    pub name: String,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    pub uri: Uri,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlbumGroup {
    Album,
    Single,
    Compilation,
    AppearsOn,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SimplifiedAlbum {
    /// The type of the album.
    pub album_type: AlbumType,
    /// The number of tracks in the album.
    pub total_tracks: usize,
    // TODO: Use a list of enums??
    /// The markets in which the album is available: [ISO 3166-1 alpha-2 country codes](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). _**NOTE:**_ an album is considered available in a market when at least 1 of its tracks is available in that market.
    pub available_markets: Vec<String>,
    /// Known external URLs for this album.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the album.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the album.
    pub id: String,
    /// The cover art for the album in various sizes, widest first.
    pub images: Vec<Image>,
    /// The name of the album. In case of an album takedown, the value may be an empty string.
    pub name: String,

    /// The date the album was first released.
    #[serde(flatten)]
    pub release: ReleaseDate,
    
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the album.
    pub uri: Uri,

    /// The artists of the album. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
    /// This field describes the relationship between the artist and the album.
    pub album_group: AlbumGroup,
    /// Not documented in official Spotify docs, however most albums do contain this field
    pub label: Option<String>,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(tag="release_date_precision", content="release_date", rename_all="snake_case")]
pub enum ReleaseDate {
    #[serde(deserialize_with = "deserialize_date_ymd")]
    Day(NaiveDate),
    #[serde(deserialize_with = "deserialize_date_ym")]
    Month(NaiveDate),
    #[serde(deserialize_with = "deserialize_date_y")]
    Year(NaiveDate),
}

impl Debug for ReleaseDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Day(d) => write!(f, "{}", d.format("%Y-%m-%d")),
            Self::Month(d) => write!(f, "{}", d.format("%Y-%m")),
            Self::Year(d) => write!(f, "{}", d.format("%Y")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Album {
    /// The type of the album.
    pub album_type: AlbumType,
    /// The number of tracks in the album.
    pub total_tracks: usize,
    // TODO: Use a list of enums??
    /// The markets in which the album is available: [ISO 3166-1 alpha-2 country codes](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). _**NOTE:**_ an album is considered available in a market when at least 1 of its tracks is available in that market.
    #[serde(default="Vec::new")]
    pub available_markets: Vec<String>,
    /// Known external URLs for this album.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the album.
    pub href: String,
    /// The Spotify ID for the album.
    pub id: String,
    /// The cover art for the album in various sizes, widest first.
    pub images: Vec<Image>,
    /// The name of the album. In case of an album takedown, the value may be an empty string.
    pub name: String,

    /// The date the album was first released.
    #[serde(flatten)]
    pub release: ReleaseDate,
    
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the album.
    pub uri: Uri,
    /// The artists of the album. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
    /// Not documented in official Spotify docs, however most albums do contain this field
    pub label: Option<String>

}

//#[derive(Debug, Clone, Deserialize, PartialEq)]
//pub struct NewReleases {
//    pub albums: Albums,
//}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NewReleases {
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the next page of items. ( null if none)
    pub next: Option<String>,
    /// URL to the previous page of items. ( null if none)
    pub previous: Option<String>,
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Album>,
}

pub trait IntoUserTopItemType {
    fn into_top_item_type() -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Artist {
    /// Known external URLs for this artist.
    pub external_urls: ExternalUrls,
    /// Information about the followers of the artist.
    pub followers: Followers,
    /// A list of the genres the artist is associated with. If not yet classified, the array is empty.
    #[serde(default="Vec::new")]
    pub genres: Vec<String>,
    /// A link to the Web API endpoint providing full details of the artist.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    pub id: String,
    /// Images of the artist in various sizes, widest first.
    #[serde(default="Vec::new")]
    pub images: Vec<Image>,
    /// The name of the artist.
    pub name: String,
    /// The popularity of the artist. The value will be between 0 and 100, with 100 being the most popular. The artist's popularity is calculated from the popularity of all the artist's tracks.
    pub popularity: u8,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    pub uri: Uri,
}
impl IntoUserTopItemType for Artist {
    fn into_top_item_type() -> &'static str {
        "artists"
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExternalIds {
    /// The [International Standard Recording Code](http://en.wikipedia.org/wiki/International_Standard_Recording_Code)
    pub isrc: Option<String>,
    /// [International Article Number](http://en.wikipedia.org/wiki/International_Article_Number)
    pub ean: Option<String>,
    /// [Universal Product Code](http://en.wikipedia.org/wiki/Universal_Product_Code)
    pub upc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Track {
    /// The album on which the track appears. The album object includes a link in href to full information about the album.
    pub album: Album,
    /// The artists who performed the track. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
    /// A list of countries in which the track can be played, identified by their [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
    #[serde(default="Vec::new")]
    pub available_markets: Vec<String>,
    /// The disc number (usually 1 unless the album consists of more than one disc).
    pub disc_number: u8,
    /// The track length in milliseconds.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the track has explicit lyrics ( true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external IDs for the track.
    pub external_ids: ExternalIds,
    /// Known external URLs for this track.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the track.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub id: String,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied. If true, the track is playable in the given market. Otherwise false.
    #[serde(default="bool::default")]
    pub is_playable: bool,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied, and the requested track has been replaced with different track. The track in the linked_from object contains information about the originally requested track.
    pub linked_from: Option<Box<Track>>,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The name of the track.
    pub name: String,
    /// A link to a 30 second preview (MP3 format) of the track.
    pub preview_url: Option<String>,
    /// The number of the track. If an album has several discs, the track number is the number on the specified disc.
    pub track_number: u8,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub uri: Uri,
    /// Whether or not the track is from a local file.
    pub is_local: bool,
}
impl IntoUserTopItemType for Track {
    fn into_top_item_type() -> &'static str {
        "tracks"
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TopItems<T: Debug + Clone + PartialEq> {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the next page of items. ( null if none)
    pub next: Option<String>,
    /// URL to the previous page of items. ( null if none)
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Cursor {
    /// The cursor to use as key to find the next page of items.
    pub after: Option<String>,
    /// The cursor to use as key to find the previous page of items.
    pub before: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FollowedArtists {
    /// A link to the Web API endpoint returning the full result of the request.
    pub href: String,
    ///The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The cursors used to find the next set of items.
    pub cursors: Cursor,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Artist>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AlbumTracks {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SimplifiedTrack>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SavedAlbum {
    /// The date and time the album was saved Timestamps are returned in ISO 8601 format as Coordinated Universal Time (UTC) with a zero offset: YYYY-MM-DDTHH:MM:SSZ.
    #[serde(deserialize_with = "deserialize_added_at")]
    pub added_at: DateTime<Local>,
    pub album: Album,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SavedAlbums {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SavedAlbum>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ArtistAlbums {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SimplifiedAlbum>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CopyRight {
    /// The copyright text for this content.
    pub text: String,
    /// The type of copyright: [C] = the copyright, [P] = the sound recording (performance) copyright.
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct ResumePoint {
    /// Whether or not the episode has been fully played by the user.
    pub fully_played: bool,
    /// The user's most recent position in the episode in milliseconds.
    #[serde(deserialize_with = "deserialize_duration")]
    pub resume_position: chrono::Duration,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Chapter {
    /// A URL to a 30 second preview (MP3 format) of the chapter. null if not available.
    /// 
    /// # Important Policy Notes
    /// - Spotify Audio preview clips [can not be a standalone service](https://developer.spotify.com/policy/#ii-respect-content-and-creators).
    #[serde(rename = "audio_preview_url")]
    pub preview_url: Option<String>,
    /// A list of the countries in which the chapter can be played, identified by their [ISO 3166-1 alpha-2](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) code.
    #[serde(default="Vec::new")]
    pub available_markets: Vec<String>,
    /// The number of the chapter
    pub chapter_number: usize,
    /// A description of the chapter. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: String,
    /// A description of the chapter. This field may contain HTML tags.
    pub html_description: String,
    /// The chapter length.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the chapter has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// External URLs for this chapter.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the chapter.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the chapter.
    pub id: String,
    /// The cover art for the chapter in various sizes, widest first.
    #[serde(default="Vec::new")]
    pub images: Vec<Image>,
    /// True if the chapter is playable in the given market. Otherwise false.
    #[serde(default)]
    pub is_playable: bool,
    /// A list of the languages used in the chapter, identified by their ISO 639-1 code.
    #[serde(default="Vec::new")]
    pub languages: Vec<String>,
    /// The name of the chapter.
    pub name: String,
    /// The date the chapter was first released, for example "1981-12-15". Depending on the precision, it might be shown as "1981" or "1981-12".
    #[serde(flatten)]
    pub release: ReleaseDate,
    /// The user's most recent position in the chapter. Set if the supplied access token is a user token and has the scope 'user-read-playback-position'.
    #[serde(default)]
    pub resume_point: ResumePoint,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the chapter.
    pub uri: Uri,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The audiobook for which the chapter belongs.
    pub audiobook: Audiobook,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SimplifiedChapter {
    /// A URL to a 30 second preview (MP3 format) of the chapter. null if not available.
    /// 
    /// # Important Policy Notes
    /// - Spotify Audio preview clips [can not be a standalone service](https://developer.spotify.com/policy/#ii-respect-content-and-creators).
    #[serde(rename = "audio_preview_url")]
    pub preview_url: Option<String>,
    /// A list of the countries in which the chapter can be played, identified by their [ISO 3166-1 alpha-2](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) code.
    #[serde(default="Vec::new")]
    pub available_markets: Vec<String>,
    /// The number of the chapter
    pub chapter_number: usize,
    /// A description of the chapter. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: String,
    /// A description of the chapter. This field may contain HTML tags.
    pub html_description: String,
    /// The chapter length.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the chapter has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// External URLs for this chapter.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the chapter.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the chapter.
    pub id: String,
    /// The cover art for the chapter in various sizes, widest first.
    #[serde(default="Vec::new")]
    pub images: Vec<Image>,
    /// True if the chapter is playable in the given market. Otherwise false.
    #[serde(default)]
    pub is_playable: bool,
    /// A list of the languages used in the chapter, identified by their ISO 639-1 code.
    #[serde(default="Vec::new")]
    pub languages: Vec<String>,
    /// The name of the chapter.
    pub name: String,
    /// The date the chapter was first released, for example "1981-12-15". Depending on the precision, it might be shown as "1981" or "1981-12".
    #[serde(flatten)]
    pub release: ReleaseDate,
    /// The user's most recent position in the chapter. Set if the supplied access token is a user token and has the scope 'user-read-playback-position'.
    #[serde(default)]
    pub resume_point: ResumePoint,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the chapter.
    pub uri: Uri,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Chapters {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SimplifiedChapter>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Audiobook {
    /// The author(s) for the audiobook.
    #[serde(default="Vec::new", deserialize_with="deserialize_named_objects")]
    pub authors: Vec<String>,
    /// A list of the countries in which the audiobook can be played, identified by their [ISO 3166-1 alpha-2](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) code.
    #[serde(default="Vec::new")]
    pub available_markets: Vec<String>,
    /// The copyright statements of the audiobook.
    #[serde(default="Vec::new")]
    pub copyrights: Vec<CopyRight>,
    /// A description of the audiobook. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: String,
    /// A description of the audiobook. This field may contain HTML tags.
    pub html_description: String,
    /// The edition of the audiobook.
    pub edition: String,
    /// Whether or not the audiobook has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external URLs for this audiobook.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the audiobook.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the audiobook.
    pub id: String,
    /// The cover art for the audiobook in various sizes, widest first.
    pub images: Vec<Image>,
    /// A list of the languages used in the audiobook, identified by their [ISO 639](https://en.wikipedia.org/wiki/ISO_639) code.
    pub languages: Vec<String>,
    /// The media type of the audiobook.
    pub media_type: String,
    /// The name of the audiobook.
    pub name: String,
    /// The narrator(s) for the audiobook.
    #[serde(default="Vec::new", deserialize_with="deserialize_named_objects")]
    pub narrators: Vec<String>,
    /// The publisher of the audiobook.
    pub publisher: String,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the audiobook.
    pub uri: Uri,
    /// The number of chapters in this audiobook.
    #[serde(default)]
    pub total_chapters: Option<usize>,

    /// Not documented in official Spotify docs, however most audiobooks do contain this field
    pub is_externally_hosted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SavedAudiobooks {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Audiobook>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Category {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The category icon, in various sizes.
    pub icons: Vec<Image>,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the category.
    pub id: String,
    /// The name of the category.
    pub name: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Categories {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Category>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Show {
    /// A list of the countries in which the show can be played, identified by their [ISO 3166-1 alpha-2](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) code.
    pub available_markets: Vec<String>,
    /// The copyright statements of the show.
    pub copyrights: Vec<CopyRight>,
    /// A description of the show. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: String,
    /// A description of the show. This field may contain HTML tags.
    pub html_description: String,
    /// Whether or not the show has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external URLs for this show.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the show.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the show.
    pub id: String,
    /// The cover art for the show in various sizes, widest first.
    pub images: Vec<Image>,
    /// True if all of the shows episodes are hosted outside of Spotify's CDN.
    #[serde(default)]
    pub is_externally_hosted: bool,
    /// A list of the languages used in the show, identified by their [ISO 639](https://en.wikipedia.org/wiki/ISO_639) code.
    #[serde(default="Vec::new")]
    pub languages: Vec<String>,
    /// The media type of the show.
    pub media_type: String,
    /// The name of the show.
    pub name: String,
    /// The publisher of the show.
    pub publisher: Option<String>,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the show.
    pub uri: Uri,
    /// The total number of episodes in this show.
    #[serde(default)]
    pub total_episodes: usize,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Episode {
    /// A URL to a 30 second preview (MP3 format) of the episode.
    #[serde(rename = "audio_preview_url")]
    pub preview_url: Option<String>,
    /// A description of the episode. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: String,
    /// A description of the episode. This field may contain HTML tags.
    pub html_description: String,
    /// The episode length.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the episode has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// External URLs for this episode.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the episode.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the episode.
    pub id: String,
    /// The cover art for the episode in various sizes, widest first.
    pub images: Vec<Image>,
    /// True if the episode is playable in the given market. Otherwise false.
    #[serde(default)]
    pub is_playable: bool,
    /// True if the episode is hosted outside of Spotify's CDN.
    #[serde(default)]
    pub is_externally_hosted: bool,
    /// A list of the languages used in the episode, identified by their [ISO 639](https://en.wikipedia.org/wiki/ISO_639) code.
    #[serde(default="Vec::new")]
    pub languages: Vec<String>,
    /// The name of the episode.
    pub name: String,
    /// The date the episode was first released, for example "1981-12-15". Depending on the precision, it might be shown as "1981" or "1981-12".
    #[serde(flatten)]
    pub release: ReleaseDate,
    /// The user's most recent position in the episode. Set if the supplied access token is a user token and has the scope 'user-read-playback-position'.
    #[serde(default)]
    pub resume_point: ResumePoint,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the episode.
    pub uri: Uri,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,

    /// The show on which the episode belongs.
    pub show: Show,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SavedEpisode {
    /// The date and time the episode was saved. Timestamps are returned in ISO 8601 format as Coordinated Universal Time (UTC) with a zero offset: YYYY-MM-DDTHH:MM:SSZ.
    #[serde(deserialize_with = "deserialize_added_at")]
    pub added_at: DateTime<Local>,
    pub episode: Episode,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SavedEpisodes {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SavedEpisode>,
}
