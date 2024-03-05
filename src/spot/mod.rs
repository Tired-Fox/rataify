use color_eyre::{Report, Section};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use std::fmt::Debug;

mod api;
pub mod auth;

pub use crate::spot::api::SpotifyRequest;
pub use api::{player::AdditionalTypes, Spotify};

#[derive(Debug, Deserialize)]
struct ErrorData {
    pub code: u16,
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
    Unknown(String),
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
        }
    }
}

impl From<Report> for Error {
    fn from(value: Report) -> Self {
        Error::Unknown(value.to_string())
    }
}

pub trait SpotifyResponse<T> {
    async fn to_spotify_response(self) -> Result<T, Error>;
}

#[derive(Debug)]
pub struct NoContent;
impl<'de> Deserialize<'de> for NoContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(match value {
            Value::Object(obj) if obj.len() == 0 => NoContent,
            Value::Null => NoContent,
            _ => return Err(serde::de::Error::custom("Expected no content")),
        })
    }
}

impl<T> SpotifyResponse<T> for Result<reqwest::Response, reqwest::Error>
where
    T: Deserialize<'static> + Debug,
{
    async fn to_spotify_response(self) -> Result<T, Error> {
        match self {
            Ok(response) => match response.status().clone() {
                StatusCode::OK => {
                    let mut body = response
                        .text()
                        .await
                        .map_err(|e| Error::Unknown(e.to_string()))?;

                    if body.is_empty() {
                        body = String::from("null");
                    }

                    Ok(serde_json::from_str::<T>(Box::leak(body.into_boxed_str()))
                        .map_err(|e| Error::Unknown(e.to_string()))?)
                }
                StatusCode::NO_CONTENT => Err(Error::NoContent),
                StatusCode::UNAUTHORIZED => Err(Error::InvalidToken),
                StatusCode::NOT_FOUND => Err(Error::NoDevice),
                StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_REQUEST => {
                    let body = response
                        .text()
                        .await
                        .map_err(|e| Error::Unknown(e.to_string()))?;
                    let body = serde_json::from_str::<ErrorBody>(&body)
                        .map_err(|e| Error::Unknown(e.to_string()))?;
                    Err(Error::Failed {
                        code: body.error.code,
                        message: body.error.message,
                    })
                }
                _ => Err(Error::Unknown("Unkown spotify response".to_string())),
            },
            Err(e) => {
                return Err(Error::Unknown(e.to_string()));
            }
        }
    }
}

// #[macro_export]
// macro_rules! scopes {
//     ($($scope: ident),* $(,)?) => {
//         std::collections::HashSet::from_iter(vec![$(stringify!($scope).replace("_", "-"),)*])
//     };
//     ($scopes: literal) => {
//         std::collections::HashSet::from_iter($scopes.split(" ").map(|s| s.to_string()))
//     };
// }
use crate::scopes;
