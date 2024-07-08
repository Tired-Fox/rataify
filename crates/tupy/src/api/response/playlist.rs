use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer};
use crate::impl_paged;
use crate::api::Uri;

use super::{deserialize_added_at_opt, ExternalUrls, Followers, Image, Item};

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TracksLink {
    /// A link to the Web API endpoint where full details of the playlist's tracks can be retrieved.
    pub href: String,
    /// Number of tracks in the playlist.
    pub total: usize,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Owner {
    /// Known public external URLs for this user.
    pub external_urls: ExternalUrls,
    /// Information about the followers of this user.
    pub followers: Option<Followers>,
    /// A link to the Web API endpoint for this user.
    pub href: String,
    /// The [Spotify user ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the user.
    pub id: String,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the user.
    pub uri: Uri,
    /// The name displayed on the user's profile. null if not available.
    #[serde(rename = "display_name")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct PlaylistItemInfo {
    /// The date and time the track or episode was added. Note: some very old playlists may return null in this field
    #[serde(deserialize_with = "deserialize_added_at_opt")]
    pub added_at: Option<DateTime<Local>>,
    /// The Spotify user who added the track or episode. Note: some very old playlists may return null in this field.
    pub added_by: Option<Owner>,
    /// Whether this track or episode is a [local file](https://developer.spotify.com/documentation/web-api/concepts/playlists#local-files) or not.
    pub  is_local: bool,
    #[serde(rename = "track")]
    pub item: Item,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct PlaylistItems {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// The number of items returned in the response
    pub total: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The requested content
    pub items: Vec<PlaylistItemInfo>,
}
impl_paged!(PlaylistItems<PlaylistItemInfo>);

fn deserialize_playlist_tracks<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where D: Deserializer<'de>
{
    let tracks = PlaylistItems::deserialize(deserializer)?;
    Ok(tracks.total)
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Playlist {
    /// true if the owner allows other users to modify the playlist.
    #[serde(default)]
    pub collaborative: bool,
    /// The playlist description. Only returned for modified, verified playlists, otherwise null.
    pub description: Option<String>,
    /// Known external URLs for this playlist.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the playlist.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the playlist.
    pub id: String,
    /// Images for the playlist. The array may be empty or contain up to three images. The images are returned by size in descending order. See Working with Playlists. Note: If returned, the source URL for the image (url) is temporary and will expire in less than a day.
    pub images: Option<Vec<Image>>,
    /// The name of the playlist.
    pub name: String,
    /// The user who owns the playlist
    pub owner: Owner,
    /// The playlist's public/private status (if it is added to the user's profile): true the playlist is public, false the playlist is private, null the playlist status is not relevant. For more about public/private status, see Working with Playlists
    pub public: Option<bool>,
    /// The version identifier for the current playlist. Can be supplied in other requests to target a specific playlist version
    #[serde(rename = "snapshot_id")]
    snapshot: String,
    ///The Spotify URI for the playlist.
    pub uri: String,
    /// The total number of items in the playlist.
    /// TODO: Check if it is reasonable to add a Pagination struct here
    #[serde(rename="tracks", deserialize_with = "deserialize_playlist_tracks")]
    pub total_items: usize,
}


#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SimplifiedPlaylist {
    /// true if the owner allows other users to modify the playlist.
    #[serde(default)]
    pub collaborative: bool,
    /// The playlist description. Only returned for modified, verified playlists, otherwise null.
    pub description: Option<String>,
    /// Known external URLs for this playlist.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the playlist.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the playlist.
    pub id: String,
    /// Images for the playlist. The array may be empty or contain up to three images. The images are returned by size in descending order. See Working with Playlists. Note: If returned, the source URL for the image (url) is temporary and will expire in less than a day.
    pub images: Option<Vec<Image>>,
    /// The name of the playlist.
    pub name: String,
    /// The user who owns the playlist
    pub owner: Owner,
    /// The playlist's public/private status (if it is added to the user's profile): true the playlist is public, false the playlist is private, null the playlist status is not relevant. For more about public/private status, see Working with Playlists
    pub public: Option<bool>,
    /// The version identifier for the current playlist. Can be supplied in other requests to target a specific playlist version
    #[serde(rename = "snapshot_id")]
    snapshot: String,
    /// A collection containing a link ( href ) to the Web API endpoint where full details of the playlist's tracks can be retrieved, along with the total number of tracks in the playlist. Note, a track object may be null. This can happen if a track is no longer available.
    pub tracks: Option<TracksLink>,
    ///The Spotify URI for the playlist.
    pub uri: Uri,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct PagedPlaylists {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// The number of items returned in the response
    pub total: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The requested content
    pub items: Vec<SimplifiedPlaylist>,
}
impl_paged!(PagedPlaylists<SimplifiedPlaylist>);

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FeaturedPlaylists {
    /// The localized message of a playlist.
    pub message: String,
    pub playlists: PagedPlaylists,
}
