use serde::Deserialize;
use crate::impl_paged;
use crate::api::Uri;

use super::{Restrictions, ReleaseDate, ResumePoint, Image, CopyRight, ExternalUrls, deserialize_duration, deserialize_named_objects};

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
impl_paged!(Chapters<SimplifiedChapter>);

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
impl_paged!(SavedAudiobooks<Audiobook>);
