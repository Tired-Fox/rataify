use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;

use color_eyre::eyre::OptionExt;
use hyper::body::Body;
use reqwest::{IntoUrl, StatusCode};
use serde::Deserialize;

pub use auth::{Callback, OAuth};
pub use credentials::Credentials;

pub use crate::browser;
use crate::error::Error;
use crate::logging::ResponseLogger;
pub use crate::query;
pub use crate::scopes;
pub use crate::spotify::api::{body, response};
use crate::spotify::api::{NoContent, SpotifyRequest, SpotifyResponse};
use crate::spotify::response::Repeat;

pub mod api;
pub mod auth;
mod cache;
mod credentials;

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

pub struct Spotify {
    pub oauth: OAuth,
    pub user: response::User,
}

impl Spotify {
    pub async fn new() -> crate::error::Result<Self> {
        // Setup OAuth and ensure that the access token is ready
        let mut oauth = OAuth::new(
            Credentials::from_env()
                .ok_or_eyre("Missing spotify id and secret environment variables")
                .map_err(Error::custom)?,
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

        match fetch_user_profile(&mut oauth).await {
            SpotifyResponse::Ok(user) => Ok(Self { user, oauth }),
            _ => Err(Error::custom("Failed to fetch user profile")),
        }
    }
}

async fn fetch_user_profile(oauth: &mut OAuth) -> SpotifyResponse<response::User> {
    let response = SpotifyRequest::get("/me").send(oauth).await;

    SpotifyResponse::from_response(response).await
}

impl Spotify {
    /// Get list of available devices
    pub async fn devices(&mut self) -> SpotifyResponse<response::Devices> {
        let response = SpotifyRequest::get("/me/player/devices")
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    /// Get current playback state
    ///
    /// Also update device in cache based on response
    pub async fn playback(&mut self) -> SpotifyResponse<response::Playback> {
        let response = SpotifyRequest::get("/me/player")
            .param("additional_types", "track,episode")
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    /// Transfer playback to a different device
    pub async fn transfer_playback(
        &mut self,
        body: &body::TransferPlayback,
    ) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::put("/me/player")
            .with_json(body)
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    /// Skip to next
    pub async fn next(&mut self) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::post("/me/player/next")
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    /// Skip to previous
    pub async fn previous(&mut self) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::post("/me/player/previous")
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    pub async fn pause(&mut self) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::put("/me/player/pause")
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    pub async fn play(&mut self, body: &body::StartPlayback) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::put("/me/player/play")
            // Start playback at
            .with_json(body)
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    pub async fn shuffle(&mut self, shuffle: bool) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::put("/me/player/shuffle")
            .param("state", shuffle)
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    pub async fn repeat(&mut self, repeat: Repeat) -> SpotifyResponse<NoContent> {
        let response = SpotifyRequest::put("/me/player/repeat")
            .param("state", format!("{}", repeat))
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }

    pub async fn queue(&mut self) -> SpotifyResponse<response::Queue> {
        let response = SpotifyRequest::get("/me/player/queue")
            .send(&mut self.oauth)
            .await;

        SpotifyResponse::from_response(response).await
    }
}

