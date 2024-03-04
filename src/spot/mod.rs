use std::fmt::Debug;
use color_eyre::{Report, Section};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use auth::Credentials;
use auth::OAuth;

use crate::scopes;

pub mod auth;
mod api;

pub use api::{Spotify, player::AdditionalTypes};
use crate::spot::api::SpotifyRequest;

#[derive(Debug, Deserialize)]
struct ErrorData {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    pub error: ErrorData
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


pub trait SpotifyResponse<T> {
    async fn to_spotify_response(self) -> Result<T, Error>;
}

impl SpotifyResponse<()> for Result<reqwest::Response, reqwest::Error> {
    async fn to_spotify_response(self) -> Result<(), Error> {
        match self {
            Ok(response) => {
                match response.status().clone() {
                    StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
                    StatusCode::UNAUTHORIZED => Err(Error::InvalidToken),
                    StatusCode::NOT_FOUND => Err(Error::NoDevice),
                    StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_REQUEST => {
                        let body = response.text().await.map_err(|e| Error::Unknown(e.to_string()))?;
                        let body = serde_json::from_str::<ErrorBody>(&body).map_err(|e| Error::Unknown(e.to_string()))?;
                        Err(Error::Failed {
                            code: body.error.code,
                            message: body.error.message
                        })
                    }
                }
            },
            Err(e) => {
                return Err(Error::Unknown(e.to_string()));
            }
        }
    }
}


impl<T> SpotifyResponse<T> for Result<reqwest::Response, reqwest::Error>
    where
        T: Deserialize<'static> + Debug
{
    async fn to_spotify_response(self) -> Result<T, Error> {
        match self {
            Ok(response) => {
                match response.status().clone() {
                    StatusCode::OK => {
                        let body = response.text().await.map_err(|e| Error::Unknown(e.to_string()))?;
                        Ok(Some(serde_json::from_str::<T>(&body).map_err(|e| Error::Unknown(e.to_string()))?))
                    },
                    StatusCode::NOT_FOUND => Err(Error::NoContent),
                    StatusCode::UNAUTHORIZED => Err(Error::InvalidToken),
                    StatusCode::NOT_FOUND => Err(Error::NoDevice),
                    StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_REQUEST => {
                        let body = response.text().await.map_err(|e| Error::Unknown(e.to_string()))?;
                        let body = serde_json::from_str::<ErrorBody>(&body).map_err(|e| Error::Unknown(e.to_string()))?;
                        Err(Error::Failed {
                            code: body.error.code,
                            message: body.error.message
                        })
                    }
                }
            },
            Err(e) => {
                return Err(Error::Unknown(e.to_string()));
            }
        }
    }
}

async fn test() {
    let oauth = OAuth::new(Credentials::from_env().unwrap(), scopes![
        user_read_private,
    ]);

    let mut spotify = Spotify::new(oauth);

    // TODO: Throw errors up and catch them before they go all the way up
    //  handle no device by opening device select
    //  handle invalid token by refreshing
    //  handle all other known errors by showing error toast or dialog
    //  handle all other errors by throwing the rest of the way up crashing the app
    let response = spotify.player_state()
        .market("US")
        .additional_types([AdditionalTypes::Track, AdditionalTypes::Episode])
        .send()
        .await;
}