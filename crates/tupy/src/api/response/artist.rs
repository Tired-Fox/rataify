use serde::Deserialize;
use crate::impl_paged;
use crate::api::Uri;

use super::{Cursors, ExternalUrls, Followers, Image, IntoUserTopItemType, Paged, SimplifiedAlbum};

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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FollowedArtists {
    /// A link to the Web API endpoint returning the full result of the request.
    pub href: String,
    ///The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The cursors used to find the next set of items.
    pub cursors: Cursors,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Artist>,
}

impl Paged for FollowedArtists {
    type Item = Artist;

    fn items(&self) -> &Vec<Self::Item> {
        &self.items
    }

    fn page(&self) -> usize {
        1
    }

    fn max_page(&self) -> usize {
        if self.total == 0 {
            1
        } else {
            (self.total as f32 / self.limit as f32).ceil() as usize
        }
    }

    fn offset(&self) -> usize {
       0 
    }

    fn limit(&self) -> usize {
        self.limit
    }

    fn total(&self) -> usize {
        self.total
    }

    fn next(&self) -> Option<&str> {
        self.next.as_deref()
    }

    fn prev(&self) -> Option<&str> {
        None
    }
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
impl_paged!(ArtistAlbums<SimplifiedAlbum>);
