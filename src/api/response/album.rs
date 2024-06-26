use serde::Deserialize;
use crate::impl_paged;

use super::{deserialize_added_at, SimplifiedTrack, ExternalUrls, Image, ReleaseDate, Restrictions, SimplifiedArtist, Uri};
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlbumGroup {
    Album,
    Single,
    Compilation,
    AppearsOn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
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
impl_paged!(AlbumTracks<SimplifiedTrack>);

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
impl_paged!(SavedAlbums<SavedAlbum>);

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
impl_paged!(NewReleases<Album>);
