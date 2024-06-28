use std::{fmt::Display, future::Future, pin::Pin, string::FromUtf8Error, sync::{Arc, Mutex}};

use reqwest::StatusCode;

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
    async fn next(&mut self) -> Result<Option<Self::Item>, Error>;
    #[allow(async_fn_in_trait)]
    async fn prev(&mut self) -> Result<Option<Self::Item>, Error>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpotifyErrorType {
    Token,
    OAuth,
    RateLimit,
    Other(u16)
}

impl From<StatusCode> for SpotifyErrorType {
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::UNAUTHORIZED => Self::Token,
            StatusCode::FORBIDDEN => Self::OAuth,
            StatusCode::TOO_MANY_REQUESTS => Self::RateLimit,
            _ => Self::Other(value.as_u16())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Other(String),
    TokenRefresh {
        message: String,
        redirect: String,
        state: String,
    },
    ScopesNotGranted(Vec<String>),
    InvalidArgument(&'static str, String),
    Auth {
        code: u16,
        error: String,
        message: String,
    },
    Request {
        error_type: SpotifyErrorType,
        code: u16,
        message: String,
    }
}

impl Error {
    pub fn custom<S: Display>(msg: S) -> Self {
        Error::Other(msg.to_string())
    }
    pub fn refresh<S: Display>(msg: S, redirect: String, state: String) -> Self {
        Error::TokenRefresh { message: msg.to_string(), redirect, state }
    }
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::Other(msg) => msg.clone(),
            Error::TokenRefresh { message, .. } => format!("failed to refresh spotify auth access token: {message}"),
            Error::InvalidArgument(name, msg) => format!("invalid argument '{}': {}", name, msg),
            Error::ScopesNotGranted(scopes) => format!(
                "the following scopes are required but not granted: {}",
                scopes.join(", ")
            ),
            Error::Auth { message, .. } => message.clone(),
            Error::Request { message, .. } => message.clone(),
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

impl From<serde_path_to_error::Error<serde_json::Error>> for Error {
    fn from(err: serde_path_to_error::Error<serde_json::Error>) -> Self {
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
