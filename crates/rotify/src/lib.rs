use color_eyre::{Report, Section};
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::fmt::Debug;
use std::future::Future;
use std::path::PathBuf;

mod api;
pub mod auth;
pub mod model;
mod prompt;

pub use api::{player::AdditionalTypes, Spotify, SpotifyRequest};
pub use model::paginate::AsyncIter;

lazy_static::lazy_static! {
    pub static ref CONFIG_PATH: PathBuf = {
        #[cfg(windows)]
        return home::home_dir().unwrap().join(".rotify");
        #[cfg(not(windows))]
        return home::home_dir().unwrap().join(".config/rotify");
    };
}


#[derive(Debug, Deserialize)]
struct ErrorData {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    pub error: ErrorData,
}

#[derive(Debug, Clone)]
pub enum Error {
    NoDevice,
    InvalidToken,
    NoContent,
    Unauthorized,
    Failed { code: u16, message: String },
    Json(String),
    Unknown(String),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value.to_string())
    }
}

impl From<Error> for Report {
    fn from(value: Error) -> Self {
        match value {
            Error::Unknown(e) => Report::msg(e),
            Error::Failed { code, message } => Report::msg(format!("{}: {}", code, message)),
            Error::Unauthorized => Report::msg("Unauthorized")
                .suggestion("Try adding the missing scope to the OAuth token"),
            Error::InvalidToken => Report::msg("Invalid token")
                .suggestion("The token is invalid or expired, try refreshing it"),
            Error::NoDevice => Report::msg("No device")
                .suggestion("No active device, try selecting one or starting playback on one"),
            Error::NoContent => Report::msg("No content in response when it was expected")
                .suggestion("Check the Spotify Web API for what a potential 204 response means"),
            Error::Json(e) => Report::msg(e),
        }
    }
}

impl From<Report> for Error {
    fn from(value: Report) -> Self {
        Error::Unknown(value.to_string())
    }
}

pub trait SpotifyResponse<T> {
    fn to_spotify_response(self) -> impl Future<Output=Result<T, Error>>;
}

#[derive(Debug)]
pub struct NoContent;

impl<'de> Deserialize<'de> for NoContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Object(map) if map.len() == 0 => Ok(NoContent),
            Value::Null => Ok(NoContent),
            _ => Err(serde::de::Error::custom("Content in response when it was not expected")),
        }
    }
}

impl<F, T> SpotifyResponse<T> for F
    where
        T: Deserialize<'static> + Debug,
        F: Future<Output=Result<reqwest::Response, reqwest::Error>>
{
    async fn to_spotify_response(self) -> Result<T, Error> {
        match self.await {
            Ok(response) => match response.status().clone() {
                StatusCode::OK => {
                    let mut body = response
                        .text()
                        .await
                        .map_err(|e| Error::Unknown(e.to_string()))?;

                    if body.is_empty() {
                        body = String::from("null");
                    }

                    let jd = &mut serde_json::Deserializer::from_str(Box::leak(body.into_boxed_str()));
                    Ok(serde_path_to_error::deserialize(jd)
                        .map_err(|e| Error::Json(e.to_string()))?)
                }
                StatusCode::NO_CONTENT => {
                    let mut body = response.text().await.map_err(|e| Error::Unknown(e.to_string()))?;
                    if body.is_empty() {
                        body = String::from("null");
                    }

                    let jd = &mut serde_json::Deserializer::from_str(Box::leak(body.into_boxed_str()));
                    match serde_path_to_error::deserialize(jd) {
                        Ok(v) => Ok(v),
                        Err(_) => Err(Error::NoContent),
                    }
                }
                StatusCode::UNAUTHORIZED => Err(Error::InvalidToken),
                StatusCode::NOT_FOUND => Err(Error::NoDevice),
                StatusCode::FORBIDDEN => Err(Error::Unauthorized),
                StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_REQUEST => {
                    let code = response.status().as_u16();
                    let body = response
                        .text()
                        .await
                        .map_err(|e| Error::Unknown(e.to_string()))?;
                    let body = serde_json::from_str::<ErrorBody>(&body)
                        .map_err(|e| Error::Json(e.to_string())).unwrap_or(ErrorBody { error: ErrorData { status: code, message: Default::default() } });
                    Err(Error::Failed {
                        code: body.error.status,
                        message: body.error.message,
                    })
                }
                code => {
                    eprintln!("{code:?}");
                    Err(Error::Unknown("Unkown spotify response".to_string()))
                }
            }
            Err(e) => {
                return Err(Error::Unknown(e.to_string()));
            }
        }
    }
}

#[macro_export]
macro_rules! scopes {
    ($($scope: ident),* $(,)?) => {
        std::collections::HashSet::from_iter(vec![$(stringify!($scope).replace("_", "-"),)*])
    };
    ($scopes: literal) => {
        std::collections::HashSet::from_iter($scopes.split(" ").map(|s| s.to_string()))
    };
}
#[macro_export]
macro_rules! browser {
    ($base: literal ? $($param: ident = $value: expr),* $(,)?) => {
        open::that(format!("{}?{}",
            $base,
            vec![
                $(format!("{}={}", stringify!($param), $value)),*
            ].join("&")
        ))
    };
    ($base: literal) => {
        open::that($base)
    };
}
