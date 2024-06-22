use std::{fmt::Display, future::Future, pin::Pin, string::FromUtf8Error, sync::{Arc, Mutex}};

pub mod api;

pub type Shared<T> = Arc<T>;
pub type Locked<T> = Mutex<T>;
pub type Pinned<T> = Pin<Box<dyn Future<Output = T>>>;

#[macro_export]
macro_rules! pinned {
    ($($r:tt)*) => {
        Box::pin(async move {
            $($r)*
        })
    };
}

/// `async fn next` using AFIT
pub trait Pagination {
    type Item;

    #[allow(async_fn_in_trait)]
    async fn next(&mut self) -> Option<(usize, Self::Item)>;
    #[allow(async_fn_in_trait)]
    async fn prev(&mut self) -> Option<(usize, Self::Item)>;
}

#[derive(Debug, Clone)]
pub enum Error {
    Other(String),
    ScopesNotGranted(Vec<String>),
    SpotifyAuth {
        code: u16,
        error: String,
        message: String,
    },
    SpotifyRequest {
        code: u16,
        message: String,
    }
}

impl Error {
    pub fn custom<S: Display>(msg: S) -> Self {
        Error::Other(msg.to_string())
    }
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::Other(msg) => msg.clone(),
            Error::ScopesNotGranted(scopes) => format!(
                "The following scopes are required but not granted: {}",
                scopes.join(", ")
            ),
            Error::SpotifyAuth { message, .. } => message.clone(),
            Error::SpotifyRequest { message, .. } => message.clone(),
        })
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::custom(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::custom(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::custom(err)
    }
}

impl From<hyper::http::uri::InvalidUri> for Error {
    fn from(err: hyper::http::uri::InvalidUri) -> Self {
        Self::custom(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::custom(err)
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self::custom(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::custom(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::custom(err)
    }
}
