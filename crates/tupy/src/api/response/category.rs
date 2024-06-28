use serde::Deserialize;
use crate::impl_paged;

use super::Image;


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
impl_paged!(Categories<Category>);
