use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Report;
use http_body_util::{Empty, Full};
use hyper::{Request, Response};
use hyper::body::{Body, Bytes, Incoming};
use hyper_util::rt::TokioIo;
use reqwest::{IntoUrl, RequestBuilder, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpStream;

pub mod auth;
mod credentials;
pub mod api;
mod cache;

pub use auth::{Callback, OAuth};
pub use credentials::Credentials;


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
pub use crate::browser;
use crate::CONFIG_PATH;
pub use crate::scopes;
pub use crate::query;
use crate::spotify::api::{Device, Playback, User};
use crate::spotify::cache::DeviceCache;

/// Helper wrapper around reqwest::Client to prefix urls with base spotify api url
struct Client(reqwest::Client);

impl Client {
    pub fn get<S: Display>(&self, url: S) -> RequestBuilder {
        self.0.get(format!("https://api.spotify.com/v1{}", url))
    }

    pub fn post<S: Display>(&self, url: S) -> RequestBuilder {
        self.0.get(format!("https://api.spotify.com/v1{}", url))
    }

    pub fn put<S: Display>(&self, url: S) -> RequestBuilder {
        self.0.put(format!("https://api.spotify.com/v1{}", url))
    }
}

pub struct Spotify {
    http_client: Client,
    oauth: OAuth,
    pub device: DeviceCache,
    pub user: User,
}

impl Spotify {
    pub async fn new() -> color_eyre::Result<Self> {
        // Setup OAuth and ensure that the access token is ready
        let mut oauth = OAuth::new(
            Credentials::from_env().ok_or_eyre("Missing spotify id and secret environment variables")?,
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
        oauth.update().await?;

        let client = Client(reqwest::Client::new());

        let mut spotify = Self {
            user: fetch_user_profile(&oauth, &client).await?,
            device: DeviceCache::load().unwrap_or_default(),
            oauth,
            http_client: client,
        };

        Ok(spotify)
    }

    /// Helper to create a `GET` request, prefixing the url with spotify api base url and
    /// automatically refreshing and adding the access token to the headers.
    fn get<S: Display>(&self, url: S) -> RequestBuilder {
        self.http_client.get(url)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
    }

    /// Helper to create a `POST` request, prefixing the url with spotify api base url and
    /// automatically refreshing and adding the access token to the headers.
    fn post<S: Display>(&self, url: S) -> RequestBuilder {
        self.http_client.post(url)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
    }

    /// Helper to create a `POST` request, prefixing the url with spotify api base url and
    /// automatically refreshing and adding the access token to the headers.
    fn put<S: Display>(&self, url: S) -> RequestBuilder {
        self.http_client.put(url)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
    }
}

async fn fetch_user_profile(oauth: &OAuth, client: &Client) -> color_eyre::Result<User> {
    let response = client.get("/me")
        .header("Authorization", oauth.token.as_ref().unwrap().to_header())
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Report::msg("Failed to fetch user profile"));
    }

    Ok(response.json().await?)
}

impl Spotify {
    pub async fn devices(&mut self) -> color_eyre::Result<&Vec<Device>> {
        let response = self.get("/me/player/devices").send().await?;
        let devices: DeviceCache = response.json().await?;
        println!("{devices:#?}");

        // load devices cache and update with spotify known devices
        self.device.update(devices.drain());

        Ok(self.device.devices())
    }

    pub async fn playback(&self) -> color_eyre::Result<Playback> {
        let response = self.get("/me/player?additional_types=track,episode")
            .send().await?;

        if !response.status().is_success() {
            return Err(Report::msg("Failed to fetch playback state"));
        }

        let body = response.text().await?;

        let jd = &mut serde_json::Deserializer::from_str(&body);
        let result = serde_path_to_error::deserialize(jd);
        match result {
            Ok(playback) => Ok(playback),
            Err(error) => {
                Err(eyre!("Failed to parse playback state: {}", error))
            }
        }
    }

    pub async fn transfer_playback<S: ToString>(&self, device_id: S, play: bool) -> color_eyre::Result<()> {
        let result = self.put("/me/player")
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string(&json!({
                    "device_ids": [ device_id.to_string() ],
                    "play": play
                }))?
            )
            .send().await?;

        if !result.status().is_success() {
            return Err(Report::msg("Failed to transfer playback to different device"));
        }

        Ok(())
    }

    pub async fn play(&self) -> color_eyre::Result<()> {
        match self.device.device() {
            Some(device) => {
                let response = self.put(format!("/me/player/play?device_id={}", device.id))
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::to_string(&json!({
                            "position_ms": 0,
                        }))?
                    )
                    .send().await?;

                if response.status().is_success() {
                    Ok(())
                } else if response.status() == StatusCode::FORBIDDEN {
                    let result = response.json::<HashMap<String, api::Error>>().await?.get("error").unwrap().clone();
                    Err(Report::msg(result.message))
                } else {
                    Err(eyre!("Failed to start playback on device {}", device.name))
                }
            }
            None => {
                Err(eyre!("No device selected"))
            }
        }
    }
}
