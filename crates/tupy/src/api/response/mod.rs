mod album;
mod artist;
mod audiobook;
mod category;
mod episode;
mod search;
mod track;
mod user;
mod playlist;
mod player;

pub use album::*;
pub use artist::*;
pub use audiobook::*;
pub use category::*;
pub use episode::*;
pub use search::*;
pub use track::*;
pub use user::*;
pub use playlist::*;
pub use player::*;

use std::{fmt::Debug, rc::Rc};

use chrono::{DateTime, Local, MappedLocalTime, NaiveDate, NaiveDateTime, TimeZone};
use reqwest::Method;
use serde::{Deserialize, Deserializer};

use crate::{Error, Pagination};

use super::{flow::AuthFlow, SpotifyRequest, SpotifyResponse, Uri};

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

pub fn deserialize_named_objects<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Vec::<Author>::deserialize(deserializer)?;
    Ok(s.iter().map(|a| a.name.clone()).collect())
}

pub fn deserialize_date_ymd<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

pub fn deserialize_date_ym<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = format!("{}-01", String::deserialize(deserializer)?);
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

pub fn deserialize_date_y<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = format!("{}-01-01", String::deserialize(deserializer)?);
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

pub fn deserialize_added_at<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ").map_err(serde::de::Error::custom)?;
    Ok(Local.from_utc_datetime(&naive))
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3fZ").map_err(serde::de::Error::custom)?;
    Ok(Local.from_utc_datetime(&naive))
}

pub fn deserialize_added_at_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(match s {
        Some(s) => {
            let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ").map_err(serde::de::Error::custom)?;
            Some(Local.from_utc_datetime(&naive))
        },
        None => None 
    })
}

pub fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = i64::deserialize(deserializer)?;
    match Local.timestamp_millis_opt(s) {
        MappedLocalTime::Single(t) => Ok(t),
        MappedLocalTime::Ambiguous(t, _) => Ok(t),
        _ => Err(serde::de::Error::custom("Invalid timestamp")),
    }
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let ms = i64::deserialize(deserializer)?;
    Ok(chrono::Duration::milliseconds(ms))
}

pub fn deserialize_duration_opt<'de, D>(deserializer: D) -> Result<Option<chrono::Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<i64>::deserialize(deserializer)? {
        Some(ms) => Some(chrono::Duration::milliseconds(ms)),
        None => None,
    })
}

pub fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let ms = f64::deserialize(deserializer)?;
    Ok(chrono::Duration::seconds(ms as i64) + chrono::Duration::milliseconds((ms.fract() * 1000.0) as i64))
}

pub fn deserialize_optional_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<usize>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone)]
pub struct Paginated<R, T, F, const N: usize>
where
    F: AuthFlow,
{
    pub(crate) offset: isize,
    pub(crate) page_size: usize,
    pub(crate) total: usize,
    pub(crate) flow: F,
    pub(crate) next: Option<String>,
    pub(crate) prev: Option<String>,
    pub(crate) resolve: Rc<dyn Fn(T) -> R>
}

impl<T, R, F, const N: usize> PartialEq for Paginated<R, T, F, N>
where
    F: AuthFlow + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && self.page_size == other.page_size
            && self.flow == other.flow
            && self.next == other.next
            && self.prev == other.prev
    }
}

impl<T, R, F, const N: usize> Debug for Paginated<R, T, F, N>
where
    F: AuthFlow + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Paginated")
            .field("page", &self.offset)
            .field("page_size", &self.page_size)
            .field("total", &self.total)
            .field("flow", &self.flow)
            .field("next", &self.next)
            .field("prev", &self.prev)
            .field("resolve", &"Fn(T) -> (R, Option<String>, Option<String>)".to_string())
            .finish()
    }
}

impl<T, R, F, const N: usize> Paginated<R, T, F, N>
where
    F: AuthFlow,
{
    pub fn new<C>(flow: F, next: Option<String>, prev: Option<String>, resolve: C) -> Self
    where
        C: Fn(T) -> R + 'static
    {
        Self {
            offset: -1,
            total: 0,
            page_size:  N,
            flow,
            next,
            prev,
            resolve: Rc::new(resolve)
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
    pub fn progress(&self) -> usize {
        self.page_size() * self.page()
    }

    pub fn total(&self) -> usize {
        self.total
    }
}

impl<R, P: Deserialize<'static>, F, const N: usize> Pagination for Paginated<R, P, F, N>
where
    F: AuthFlow,
    R: Paged,
{
    type Item = R;
    async fn next(&mut self) -> Result<Option<Self::Item>, Error> {
        let next = match self.next.as_ref() {
            Some(next) => next,
            None => return Ok(None),
        };

        let SpotifyResponse { body, .. } = SpotifyRequest::new(Method::GET, next).send_raw(self.flow.token()).await?;
        let eb = body.clone();
        let body = body.into_boxed_str();
        let response = match pares!(P: Box::leak(body)) {
            Ok(item) => {
                let result = (self.resolve)(item);
                self.total = result.total();

                if result.items().is_empty() || (result.page() + result.limit() >= result.total()) {
                    self.prev = result.prev().map(|s| s.to_string());
                    self.next = None;
                } else {
                    self.prev = result.prev().map(|s| s.to_string());
                    self.next = result.next().map(|s| s.to_string());
                }

                Ok(Some(result))
            },
            Err(e) => {
                eprintln!("{eb:?}");
                Err(Error::custom(e))
            }
        };
        self.offset += 1;
        response
    }

    async fn prev(&mut self) -> Result<Option<Self::Item>, Error> {
        if self.offset < 1 {
            return Ok(None);
        }

        let prev = match self.prev.as_ref() {
            Some(prev) => prev,
            None => return Ok(None),
        };

        let SpotifyResponse { body, .. } = SpotifyRequest::new(Method::GET, prev).send_raw(self.flow.token()).await?;
        let body = body.into_boxed_str();
        let response = match pares!(P: Box::leak(body)) {
            Ok(item) => {
                let result = (self.resolve)(item);
                self.total = result.total();

                if result.items().is_empty() || (result.page() + result.limit() >= result.total()) {
                    self.prev = result.prev().map(|s| s.to_string());
                    self.next = None;
                } else {
                    self.prev = result.prev().map(|s| s.to_string());
                    self.next = result.next().map(|s| s.to_string());
                }

                Ok(Some(result))
            },
            Err(e) => Err(Error::custom(e))
        };
        self.offset -= 1;
        response
    }
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
    #[serde(default, deserialize_with = "deserialize_optional_usize")]
    pub height: usize,
    /// The image width in pixels.
    #[serde(default, deserialize_with = "deserialize_optional_usize")]
    pub width: usize,
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

impl ReleaseDate {
    pub fn unwrap(self) -> NaiveDate {
        match self {
            Self::Day(d) => d,
            Self::Month(d) => d,
            Self::Year(d) => d,
        }
    }
}

impl AsRef<NaiveDate> for ReleaseDate {
    fn as_ref(&self) -> &NaiveDate {
        match self {
            Self::Day(d) => d,
            Self::Month(d) => d,
            Self::Year(d) => d,
        }
    }
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

pub trait IntoUserTopItemType {
    fn into_top_item_type() -> &'static str;
}

pub trait Paged {
    type Item;
    fn items(&self) -> &Vec<Self::Item>;
    fn page(&self) -> usize;
    fn limit(&self) -> usize;
    fn total(&self) -> usize;
    fn next(&self) -> Option<&str>;
    fn prev(&self) -> Option<&str>;
}

#[macro_export]
macro_rules! impl_paged {
    ($name: ident<$typ: ty>) => { 
        impl $crate::api::response::Paged for $name {
            type Item = $typ;

            fn items(&self) -> &Vec<Self::Item> {
                &self.items
            }

            fn next(&self) -> Option<&str> {
                self.next.as_deref()
            }

            fn prev(&self) -> Option<&str> {
                self.previous.as_deref()
            }

            fn page(&self) -> usize {
                self.offset
            }

            fn limit(&self) -> usize {
                self.limit
            }

            fn total(&self) -> usize {
                self.total
            }
        }
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Cursors {
    /// The cursor to use as key to find the next page of items.
    pub after: Option<String>,
    /// The cursor to use as key to find the previous page of items.
    pub before: Option<String>,
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
    #[serde(rename = "resume_position_ms", deserialize_with = "deserialize_duration", default="chrono::Duration::zero")]
    pub resume_position: chrono::Duration,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    Track(Box<Track>),
    Episode(Box<Episode>),
}

impl Item {
    pub fn uri(&self) -> Uri {
        match self {
            Self::Track(t) => t.uri.clone(),
            Self::Episode(e) => e.uri.clone(),
        }
    }
}
