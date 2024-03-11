use std::collections::HashMap;
use std::fmt::Display;
use serde_json::json;

use crate::{Error, SpotifyRequest, SpotifyResponse};
use crate::auth::OAuth;
use crate::model::follow::FollowedArtists;
use crate::model::Wrapped;

pub struct FollowedArtistsBuilder<'a> {
    oauth: &'a mut OAuth,
    after: Option<String>,
    limit: Option<usize>,
}

impl<'a> FollowedArtistsBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            after: None,
            limit: None,
        }
    }

    pub fn after(mut self, after: String) -> Self {
        self.after = Some(after);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl<'a> SpotifyRequest<FollowedArtists> for FollowedArtistsBuilder<'a> {
    async fn send(self) -> Result<FollowedArtists, Error> {
        self.oauth.update().await?;

        let mut query = HashMap::from([("type", "artist".to_string())]);
        if let Some(after) = self.after {
            query.insert("after", after);
        }

        if let Some(limit) = self.limit {
            query.insert("limit", limit.to_string());
        }

        let result: Result<Wrapped<FollowedArtists>, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/following")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .query(&query)
            .send()
            .to_spotify_response()
            .await;

        result.map(|v| v.unwrap())
    }
}

pub struct FollowPlaylistBuilder<'a> {
    oauth: &'a mut OAuth,
    playlist_id: String,
    public: bool,
}

impl<'a> FollowPlaylistBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, playlist_id: String) -> Self {
        Self {
            oauth,
            playlist_id,
            public: true,
        }
    }

    pub fn public(mut self, public: bool) -> Self {
        self.public = public;
        self
    }
}

impl<'a> SpotifyRequest<()> for FollowPlaylistBuilder<'a> {
    async fn send(self) -> Result<(), Error> {
        self.oauth.update().await?;

        let body = serde_json::to_string(&json!({
            "public": self.public
        }))?;

        reqwest::Client::new()
            .put(format!("https://api.spotify.com/v1/playlists/{}/followers", self.playlist_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct UnfollowPlaylistBuilder<'a> {
    oauth: &'a mut OAuth,
    playlist_id: String,
}

impl<'a> UnfollowPlaylistBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, playlist_id: String) -> Self {
        Self {
            oauth,
            playlist_id,
        }
    }
}

impl<'a> SpotifyRequest<()> for UnfollowPlaylistBuilder<'a> {
    async fn send(self) -> Result<(), Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .delete(format!("https://api.spotify.com/v1/playlists/{}/followers", self.playlist_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}
