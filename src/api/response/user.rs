use std::fmt::Debug;

use serde::Deserialize;

use super::{ExternalUrls, Followers, Image, Paged, Uri};


/// Explicit content settings
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExplicitContent {
    /// When true, indicates that explicit content should not be played.
    pub filter_enabled: bool,
    /// When true, indicates that the explicit content setting is locked and can't be changed by the user.
    pub filter_locked: bool,
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

impl<T> Paged for TopItems<T>
where
    T: Debug + Clone + PartialEq,
{
    type Item = T;

    fn items(&self) -> &Vec<Self::Item> {
        &self.items
    }

    fn next(&self) -> Option<&str> {
        self.next.as_ref().map(|s| s.as_str())
    }

    fn prev(&self) -> Option<&str> {
        self.previous.as_ref().map(|s| s.as_str())
    }

    fn limit(&self) -> usize {
        self.limit
    }

    fn page(&self) -> usize {
        self.offset
    }

    fn total(&self) -> usize {
        self.total
    }
}
