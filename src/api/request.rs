#[macro_export]
macro_rules! spotify_request {
    ($type: ident, $url: literal) => {
        $crate::api::SpotifyRequest::$type($url)
    };
    ($type: ident, $url: literal, $($param: expr),*) => {
        $crate::api::SpotifyRequest::$type(format!($url, $($param,)*))
    }
}

#[macro_export]
macro_rules! spotify_request_get {
    ($($rest: tt)*) => {
        $crate::spotify_request!(get, $($rest)*)
    }
}

#[macro_export]
macro_rules! spotify_request_post {
    ($(rest: tt)*) => {
        $crate::spotify_request!(post, $(rest)*)
    }
}

use std::fmt::Display;
use std::fmt::Formatter;

pub use crate::spotify_request_get as get;
pub use crate::spotify_request_post as post;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    Short,
    Medium,
    Long
}

impl Display for TimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRange::Short => write!(f, "short_term"),
            TimeRange::Medium => write!(f, "medium_term"),
            TimeRange::Long => write!(f, "long_term"),
        }
    }
}
