use std::collections::HashMap;
use std::fmt::Display;

use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Report;
use hyper::body::Body;
use reqwest::{IntoUrl, RequestBuilder, StatusCode};
use serde::Deserialize;

pub use auth::{Callback, OAuth};
pub use credentials::Credentials;

pub use crate::browser;
use crate::error::Error;
pub use crate::query;
pub use crate::scopes;
pub use crate::spotify::api::{body, response};
use crate::spotify::api::SpotifyRequest;
use crate::spotify::cache::DeviceCache;
use crate::spotify::response::Devices;

pub mod auth;
mod credentials;
mod cache;
pub mod api;

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
#[macro_export]
macro_rules! query {
    ($base: literal ? $($param: ident = $value: expr),* $(,)?) => {
        format!("{}?{}",
            $base,
            vec![
                $(format!("{}={}", stringify!($param), $value)),*
            ].join("&")
        )
    };
    ($base: literal) => {
        $base.to_string()
    };
}
/// Helper wrapper around reqwest::Client to prefix urls with base spotify api url
struct Client {
    client: reqwest::Client,
    oauth: OAuth,
}

impl Client {
    pub fn new(oauth: OAuth) -> Self {
        Self {
            client: reqwest::Client::new(),
            oauth,
        }
    }

    pub async fn get<S: Display>(&mut self, url: S) -> RequestBuilder {
        self.oauth.update().await.unwrap();
        self.client.get(format!("https://api.spotify.com/v1{}", url))
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
    }

    pub async fn post<S: Display>(&mut self, url: S) -> RequestBuilder {
        self.oauth.update().await.unwrap();
        self.client.get(format!("https://api.spotify.com/v1{}", url))
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
    }

    pub async fn put<S: Display>(&mut self, url: S) -> RequestBuilder {
        self.oauth.update().await.unwrap();
        self.client.put(format!("https://api.spotify.com/v1{}", url))
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
    }
}

pub struct Spotify {
    pub oauth: OAuth,
    pub user: response::User,
}

impl Spotify {
    pub async fn new() -> crate::error::Result<Self> {
        // Setup OAuth and ensure that the access token is ready
        let mut oauth = OAuth::new(
            Credentials::from_env().ok_or_eyre("Missing spotify id and secret environment variables").map_err(Error::custom)?,
            scopes!(
                // User information permissions
                user_read_private,
                user_read_email,
                user_top_read,
                user_read_playback_state,
                user_modify_playback_state,
                user_read_currently_playing,
                user_follow_read,
                user_follow_modify,

                // Playlist permissions
                playlist_modify_public,
                playlist_modify_private,
            ),
        );

        Ok(Self {
            user: fetch_user_profile(&mut oauth).await?,
            oauth,
        })
    }
}

async fn fetch_user_profile(oauth: &mut OAuth) -> crate::error::Result<response::User> {
    let response = SpotifyRequest::get("/me")
        .send(oauth)
        .await.map_err(Error::from)?;

    if !response.status().is_success() {
        return Err(Error::custom("Failed to fetch user profile"));
    }

    Ok(response.json().await.map_err(Error::custom)?)
}

impl Spotify {
    /// Get list of available devices
    pub async fn devices(&mut self) -> crate::error::Result<Vec<response::Device>> {
        let response = SpotifyRequest::get("/me/player/devices")
            .send(&mut self.oauth)
            .await.map_err(Error::from)?;
        Ok(response.json::<Devices>().await.map_err(Error::custom)?.devices)
    }

    /// Get current playback state
    ///
    /// Also update device in cache based on response
    pub async fn playback(&mut self) -> crate::error::Result<response::Playback> {
        let response = SpotifyRequest::get("/me/player")
            .param("additional_types", "track,episode")
            .send(&mut self.oauth)
            .await.map_err(Error::from)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::NoDevice);
        } else if !response.status().is_success() {
            return Err(Error::custom("Failed to fetch playback state"));
        }

        let body = response.text().await.map_err(Error::custom)?;

        let jd = &mut serde_json::Deserializer::from_str(&body);
        Ok(serde_path_to_error::deserialize(jd).map_err(Error::custom)?)
    }

    /// Transfer playback to a different device
    pub async fn transfer_playback(&mut self, body: &body::TransferPlayback) -> crate::error::Result<()> {
        let response = SpotifyRequest::put("/me/player")
            .with_json(body).map_err(Error::from)?
            .send(&mut self.oauth)
            .await.map_err(Error::from)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::NoDevice);
        } else if !response.status().is_success() {
            return Err(Error::custom("Failed to transfer playback to different device"));
        }

        Ok(())
    }

    /// Skip to next
    pub async fn next(&mut self) -> crate::error::Result<()> {
        let response = SpotifyRequest::post("/me/player/next")
            .send(&mut self.oauth)
            .await.map_err(Error::from)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::NoDevice);
        } else if response.status().is_client_error() {
            return Err(Error::custom("Failed to skip to next"));
        }
        Ok(())
    }

    /// Skip to previous
    pub async fn previous(&mut self) -> crate::error::Result<()> {
        let response = SpotifyRequest::post("/me/player/previous")
            .send(&mut self.oauth)
            .await.map_err(Error::from)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::NoDevice);
        } else if !response.status().is_success() {
            return Err(Error::custom("Failed to skip to next"));
        }
        Ok(())
    }

    pub async fn pause(&mut self) -> crate::error::Result<()> {
        let response = SpotifyRequest::put("/me/player/pause")
            .send(&mut self.oauth)
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::NoDevice);
        } else if !response.status().is_success() {
            return Err(Error::custom("Failed to pause playback"));
        }
        Ok(())
    }

    // TODO: Update to play from currently cached device or make the user select a device
    pub async fn play(&mut self, body: &body::StartPlayback) -> crate::error::Result<()> {
        let response = SpotifyRequest::put("/me/player/play")
            // Start playback at
            .with_json(body).map_err(Error::custom)?
            .send(&mut self.oauth)
            .await.map_err(Error::from)?;

        if response.status() == StatusCode::FORBIDDEN {
            let result = response.json::<HashMap<String, response::Error>>().await.map_err(Error::custom)?.get("error").unwrap().clone();
            return Err(Error::custom(result.message));
        } else if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::NoDevice);
        } else if !response.status().is_success() {
            return Err(Error::custom("Failed to start playback"));
        }

        Ok(())
    }
}
