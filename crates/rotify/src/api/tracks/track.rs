use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use serde_json::json;
use crate::auth::OAuth;
use crate::model::player::{Track, Tracks};
use crate::{Error, query, SpotifyRequest, SpotifyResponse};
use crate::model::paginate::{Paginate, AsyncIter, PaginationIter, PaginateCursor};
use crate::model::tracks::PagedTracks;

pub struct GetTrackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    id: String,
    market: Option<String>,
}

impl GetTrackBuilder {
    pub fn new<S: Into<String>>(oauth: Arc<Mutex<OAuth>>, id: S) -> Self {
        Self {
            oauth,
            id: id.into(),
            market: None,
        }
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.market = Some(market.into());
        self
    }
}

impl SpotifyRequest for GetTrackBuilder {
    type Response = Track;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/tracks/{}", self.id))
            .header("Authorization", auth.unwrap().to_header())
            .query(&query! {
                "market": self.market.clone()
            })
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetTracksBuilder {
    oauth: Arc<Mutex<OAuth>>,
    ids: Vec<String>,
    market: Option<String>,
}

impl GetTracksBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
            market: None,
        }
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.market = Some(market.into());
        self
    }
}

impl SpotifyRequest for GetTracksBuilder {
    type Response = Tracks;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/tracks")
            .header("Authorization", auth.unwrap().to_header())
            .query(&query! {
                "ids": Some(self.ids.join(",")),
                "market": self.market.clone()
            })
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetSavedTracksBuilder {
    oauth: Arc<Mutex<OAuth>>,
    limit: Option<usize>,
    offset: Option<usize>,
    market: Option<String>,
}

impl GetSavedTracksBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self {
            oauth,
            limit: None,
            offset: None,
            market: None,
        }
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.market = Some(market.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_pagination(mut self, paginate: Paginate) -> Self {
        self.limit = paginate.limit.clone();
        self.offset = paginate.offset.clone();
        self
    }

    /// Get a two-way async iterator over the endpoints next and previous links.
    /// regardless of if `next` or `prev` are called, the first call is the initial page
    /// setup by the builders initial values. Every call after that will follow the next and previous
    /// urls respectively.
    ///
    /// # Examples
    /// ```
    /// use rotify::{AsyncIter, Spotify};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut spotify = Spotify::new();
    ///
    ///     let mut saved_tracks = spotify.tracks()
    ///         .get_saved_tracks()
    ///         .limit(50)
    ///         .iter();
    ///
    ///     // First iteration will be the result of the initial values in the builder
    ///     while let Some(Ok(saved)) = saved_tracks.next().await {
    ///         for track in saved.items.iter() {
    ///             println!("{}", track.track.name);
    ///         }
    ///
    ///         // Break after 100 tracks have been printed
    ///         if (saved.offset.unwrap_or(0) + saved.limit.unwrap_or(20)) >= 100 {
    ///             break;
    ///         }
    ///     }
    /// }
    ///
    /// ```
    pub fn iter(self) -> PaginationIter<PagedTracks, Option<String>> {
        PaginationIter::new(
            self.oauth,
            Some(self.market),
            Paginate::new(self.limit, self.offset),
            |paginate, auth, state| {
                Box::pin(async move {
                    let result: Result<PagedTracks, Error> = reqwest::Client::new()
                        .get("https://api.spotify.com/v1/me/tracks")
                        .header("Authorization", auth)
                        .query(&query! {
                            "market": state.unwrap(),
                            "limit": paginate.limit,
                            "offset": paginate.offset
                        })
                        .send()
                        .to_spotify_response()
                        .await;

                    (
                        PaginateCursor {
                            next: result.as_ref().map(|r| r.next.clone()).unwrap_or(None),
                            prev: result.as_ref().map(|r| r.previous.clone()).unwrap_or(None),
                        },
                        result
                    )
                })
            },
        )
    }
}

impl SpotifyRequest for GetSavedTracksBuilder {
    type Response = PagedTracks;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", auth.unwrap().to_header())
            .query(&query! {
                "market": self.market.clone(),
                "limit": self.limit,
                "offset": self.offset
            })
            .send()
            .to_spotify_response()
            .await
    }
}

/// Check saved tracks
pub struct CheckSavedTracksBuilder {
    oauth: Arc<Mutex<OAuth>>,
    ids: Vec<String>,
}

impl CheckSavedTracksBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl SpotifyRequest for CheckSavedTracksBuilder {
    type Response = Vec<bool>;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/tracks/contains")
            .header("Authorization", auth.unwrap().to_header())
            .query(&[("ids", self.ids.join(","))])
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct SaveTracksBuilder {
    oauth: Arc<Mutex<OAuth>>,
    ids: Vec<String>,
}

impl SaveTracksBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl SpotifyRequest for SaveTracksBuilder {
    type Response = ();

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        let data = &json!({
            "ids": self.ids
        });
        let body = serde_json::to_string(&data)?;

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", auth.unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct RemoveTracksBuilder {
    oauth: Arc<Mutex<OAuth>>,
    ids: Vec<String>,
}

impl RemoveTracksBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl SpotifyRequest for RemoveTracksBuilder {
    type Response = ();

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        let data = &json!({
            "ids": self.ids
        });

        let body = serde_json::to_string(&data)?;

        reqwest::Client::new()
            .delete("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", auth.unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}
