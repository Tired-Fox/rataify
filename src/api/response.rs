use std::fmt::Debug;

use serde::{Deserialize, Deserializer};

use crate::Pagination;
//use crate::api::datetime::{
//    deserialize_date, deserialize_datetime, NaiveDate, NaiveDateTime
//};

use super::{flow::AuthFlow, SpotifyRequest, SpotifyResponse, API_BASE_URL};

pub struct Paginated<T, F, const N: usize>
where
    F: AuthFlow,
{
    pub(crate) flow: F,
    pub(crate) next: Option<String>,
    pub(crate) prev: Option<String>,
    pub(crate) resolve: Box<dyn Fn(&T) -> (Option<String>, Option<String>)>
}

impl<T, F, const N: usize> Paginated<T, F, N>
where
    F: AuthFlow,
{
    pub fn new<C>(flow: F, next: Option<String>, prev: Option<String>, resolve: C) -> Self
    where
        C: Fn(&T) -> (Option<String>, Option<String>) + 'static
    {
        Self {
            flow,
            next,
            prev,
            resolve: Box::new(resolve)
        }
    }
}

impl<T: Deserialize<'static>, F, const N: usize> Pagination for Paginated<T, F, N>
where
    F: AuthFlow,
{
    type Item = T;
    async fn next(&mut self) -> Option<(usize, Self::Item)> {
        if self.next.is_none() {
            return None;
        }

        let next = self.next.as_ref().unwrap();
        match SpotifyRequest::get(next).send_raw(self.flow.token().await.ok()?).await {
            Ok(SpotifyResponse { body, .. }) => {
                let body = body.into_boxed_str();
                match serde_json::from_str::<Self::Item>(Box::leak(body)) {
                    Ok(item) => {
                        let (next, prev) = (self.resolve)(&item);
                        self.next = next;
                        self.prev = prev;
                        Some((0, item))
                    },
                    Err(err) => {
                        eprintln!("{:?}", err);
                        None
                    }
                }
            },
            Err(err) => {
                eprintln!("{:?}", err);
                None
            }
        }
    }

    async fn prev(&mut self) -> Option<(usize, Self::Item)> {
        if self.prev.is_none() {
            return None;
        }

        let prev = self.prev.as_ref().unwrap();
        match SpotifyRequest::get(prev).send_raw(self.flow.token().await.ok()?).await {
            Ok(SpotifyResponse { body, .. }) => {
                let body = body.into_boxed_str();
                match serde_json::from_str::<Self::Item>(Box::leak(body)) {
                    Ok(item) => {
                        let (next, prev) = (self.resolve)(&item);
                        self.next = next;
                        self.prev = prev;
                        Some((0, item))
                    },
                    Err(err) => {
                        eprintln!("{:?}", err);
                        None
                    }
                }
            },
            Err(err) => {
                eprintln!("{:?}", err);
                None
            }
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
#[derive(Debug, Clone, Deserialize, PartialEq)]
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
    /// Example: "https://i.scdn.co/image/ab67616d00001e02ff9ca10b55ce82ae553c8228"
    pub url: String,
    /// The image height in pixels.
    /// Example: 300
    pub height: u32,
    /// The image width in pixels.
    /// Example: 300
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
    pub uri: String,
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

/// Album types
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
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
pub struct SimplifiedArtist {
    /// Known external URLs for this artist.
    external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the artist.
    href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    id: String,
    /// The name of the artist.
    name: String,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    /// Example: "spotify:artist:4iV5W9uYEdYUVa79Axb7Rh"
    uri: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Album {
    /// The type of the album.
    /// Example: "compilation"
    pub album_type: AlbumType,
    /// The number of tracks in the album.
    /// Example: 9
    pub total_tracks: usize,
    // TODO: Use a list of enums??
    /// The markets in which the album is available: [ISO 3166-1 alpha-2 country codes](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). _**NOTE:**_ an album is considered available in a market when at least 1 of its tracks is available in that market.
    pub available_markets: Vec<String>,
    /// Known external URLs for this album.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the album.
    pub href: String,
    /// The Spotify ID for the album.
    /// Example: "2up3OPMp9Tb4dAKM2erWXQ"
    pub id: String,
    /// The cover art for the album in various sizes, widest first.
    images: Vec<Image>,
    /// The name of the album. In case of an album takedown, the value may be an empty string.
    pub name: String,
    /// The date the album was first released.
    /// Example: "1981-12"
    pub release_date: String,
    // The precision with which release_date value is known.
    // Example: "year"
    pub release_date_precision: DatePrecision,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the album.
    /// Example: "spotify:album:2up3OPMp9Tb4dAKM2erWXQ"
    pub uri: String,
    /// The artists of the album. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NewReleases {
    pub albums: Albums,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Albums {
    /// The maximum number of items in the response (as set in the query or by default).
    /// Example: 20
    pub limit: usize,
    /// The offset of the items returned (as set in the query or by default)
    /// Example: 0
    pub offset: usize,
    /// URL to the next page of items. ( null if none)
    /// Example: "https://api.spotify.com/v1/me/shows?offset=1&limit=1"
    pub next: Option<String>,
    /// URL to the previous page of items. ( null if none)
    /// Example: "https://api.spotify.com/v1/me/shows?offset=1&limit=1"
    pub previous: Option<String>,
    /// A link to the Web API endpoint returning the full result of the request
    /// Example: "https://api.spotify.com/v1/me/shows?offset=0&limit=20"
    pub href: String,
    /// The total number of items available to return.
    /// Example: 4
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
    pub genres: Vec<String>,
    /// A link to the Web API endpoint providing full details of the artist.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    pub id: String,
    /// Images of the artist in various sizes, widest first.
    pub images: Vec<Image>,
    /// The name of the artist.
    pub name: String,
    /// The popularity of the artist. The value will be between 0 and 100, with 100 being the most popular. The artist's popularity is calculated from the popularity of all the artist's tracks.
    pub popularity: u8,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the artist.
    /// Example: "spotify:artist:4iV5W9uYEdYUVa79Axb7Rh"
    pub uri: String,
}
impl IntoUserTopItemType for Artist {
    fn into_top_item_type() -> &'static str {
        "artists"
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExternalIds {
    /// The [International Standard Recording Code](http://en.wikipedia.org/wiki/International_Standard_Recording_Code)
    pub isrc: String,
    /// [International Article Number](http://en.wikipedia.org/wiki/International_Article_Number)
    pub ean: String,
    /// [Universal Product Code](http://en.wikipedia.org/wiki/Universal_Product_Code)
    pub upc: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Track {
    /// The album on which the track appears. The album object includes a link in href to full information about the album.
    pub album: Album,
    /// The artists who performed the track. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<Artist>,
    /// A list of countries in which the track can be played, identified by their [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
    pub available_markets: Vec<String>,
    /// The disc number (usually 1 unless the album consists of more than one disc).
    pub disc_number: u8,
    /// The track length in milliseconds.
    #[serde(rename = "duration_ms")]
    pub durations: usize,
    /// Whether or not the track has explicit lyrics ( true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external IDs for the track.
    pub external_ids: ExternalIds,
    /// Known external URLs for this track.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the track.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    /// Example: "spotify:track:4iV5W9uYEdYUVa79Axb7Rh"
    pub id: String,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied. If true, the track is playable in the given market. Otherwise false.
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
    /// Example: "spotify:track:4iV5W9uYEdYUVa79Axb7Rh"
    pub uri: String,
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
    /// Example: "https://api.spotify.com/v1/me/shows?offset=0&limit=20"
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    /// Example: 20
    pub limit: usize,
    /// The offset of the items returned (as set in the query or by default)
    /// Example: 0
    pub offset: usize,
    /// URL to the next page of items. ( null if none)
    /// Example: "https://api.spotify.com/v1/me/shows?offset=1&limit=1"
    pub next: Option<String>,
    /// URL to the previous page of items. ( null if none)
    /// Example: "https://api.spotify.com/v1/me/shows?offset=1&limit=1"
    pub previous: Option<String>,
    /// The total number of items available to return.
    /// Example: 4
    pub total: usize,
    pub items: Vec<T>,
}
