use serde::Deserialize;
use crate::impl_paged;
use crate::api::Uri;

use super::{deserialize_duration, ExternalUrls, Image, ReleaseDate, Restrictions, ResumePoint, CopyRight, deserialize_added_at};
use chrono::{Duration, DateTime, Local};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Show {
    /// A list of the countries in which the show can be played, identified by their [ISO 3166-1 alpha-2](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) code.
    pub available_markets: Vec<String>,
    /// The copyright statements of the show.
    pub copyrights: Vec<CopyRight>,
    /// A description of the show. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: Option<String>,
    /// A description of the show. This field may contain HTML tags.
    pub html_description: Option<String>,
    /// Whether or not the show has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external URLs for this show.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the show.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the show.
    pub id: String,
    /// The cover art for the show in various sizes, widest first.
    #[serde(default)]
    pub images: Vec<Image>,
    /// True if all of the shows episodes are hosted outside of Spotify's CDN.
    #[serde(default)]
    pub is_externally_hosted: Option<bool>,
    /// A list of the languages used in the show, identified by their [ISO 639](https://en.wikipedia.org/wiki/ISO_639) code.
    #[serde(default)]
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
    pub description: Option<String>,
    /// A description of the episode. This field may contain HTML tags.
    pub html_description: Option<String>,
    /// The episode length.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    /// Whether or not the episode has explicit content (true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// External URLs for this episode.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the episode.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the episode.
    pub id: String,
    /// The cover art for the episode in various sizes, widest first.
    #[serde(default)]
    pub images: Vec<Image>,
    /// True if the episode is playable in the given market. Otherwise false.
    #[serde(default)]
    pub is_playable: bool,
    /// True if the episode is hosted outside of Spotify's CDN.
    #[serde(default)]
    pub is_externally_hosted: Option<bool>,
    /// A list of the languages used in the episode, identified by their [ISO 639](https://en.wikipedia.org/wiki/ISO_639) code.
    #[serde(default)]
    pub languages: Vec<String>,
    /// The name of the episode.
    pub name: String,
    /// The date the episode was first released, for example "1981-12-15". Depending on the precision, it might be shown as "1981" or "1981-12".
    #[serde(flatten)]
    pub release: Option<ReleaseDate>,
    /// The user's most recent position in the episode. Set if the supplied access token is a user token and has the scope 'user-read-playback-position'.
    #[serde(default)]
    pub resume_point: ResumePoint,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the episode.
    pub uri: Uri,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,

    /// The show on which the episode belongs.
    pub show: Option<Show>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SimplifiedEpisode {
    /// A URL to a 30 second preview (MP3 format) of the episode.
    #[serde(rename = "audio_preview_url")]
    pub preview_url: Option<String>,
    /// A description of the episode. HTML tags are stripped away from this field, use html_description field in case HTML tags are needed.
    pub description: String,
    /// A description of the episode. This field may contain HTML tags.
    pub html_description: String,
    /// The episode length.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
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
    pub is_externally_hosted: Option<bool>,
    /// A list of the languages used in the episode, identified by their [ISO 639](https://en.wikipedia.org/wiki/ISO_639) code.
    #[serde(default)]
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
impl_paged!(SavedEpisodes<SavedEpisode>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ShowEpisodes {
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
    pub items: Vec<SimplifiedEpisode>,
}
impl_paged!(ShowEpisodes<SimplifiedEpisode>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SavedShow {
    /// The date and time the show was saved. Timestamps are returned in ISO 8601 format as Coordinated Universal Time (UTC) with a zero offset: YYYY-MM-DDTHH:MM:SSZ.
    #[serde(deserialize_with = "deserialize_added_at")]
    pub added_at: DateTime<Local>,
    pub show: Show,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SavedShows {
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
    pub items: Vec<SavedShow>,
}
impl_paged!(SavedShows<SavedShow>);
