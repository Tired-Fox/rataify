#[macro_export]
macro_rules! spotify_request {
    ($type: ident, $url: literal) => {
        paste::paste! {
            $crate::api::SpotifyRequest::<String>::new(hyper::Method::[<$type:upper>], format!($url))
        }
    };
    ($type: ident, $url: literal, $($param: expr),*) => {
        paste::paste! {
            $crate::api::SpotifyRequest::<String>::new(hyper::Method::[<$type:upper>], format!($url, $($param,)*))
        }
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

#[macro_export]
macro_rules! spotify_request_put {
    ($($rest: tt)*) => {
        $crate::spotify_request!(put, $($rest)*)
    }
}

#[macro_export]
macro_rules! spotify_request_delete {
    ($($rest: tt)*) => {
        $crate::spotify_request!(delete, $($rest)*)
    }
}

use std::fmt::Display;
use std::fmt::Formatter;

pub use crate::spotify_request_get as get;
pub use crate::spotify_request_post as post;
pub use crate::spotify_request_put as put;
pub use crate::spotify_request_delete as delete;

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
