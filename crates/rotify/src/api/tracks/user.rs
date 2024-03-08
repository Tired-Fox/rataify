use std::pin::Pin;
use std::task::{Context, Poll};

use serde_json::json;

use crate::{Error, SpotifyRequest, SpotifyResponse};
use crate::api::AsyncIter;
use crate::auth::OAuth;
use crate::model::paginate::Paginate;
use crate::model::tracks::PagedTracks;

pub struct GetSavedTracksBuilder<'a> {
    oauth: &'a mut OAuth,
    limit: Option<usize>,
    offset: Option<usize>,
    market: Option<String>,
}

impl<'a> GetSavedTracksBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
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
    pub fn iter(self) -> GetSavedTracksIter<'a> {
        GetSavedTracksIter::new(self)
    }
}

impl<'a> SpotifyRequest<PagedTracks> for GetSavedTracksBuilder<'a> {
    async fn send(mut self) -> Result<PagedTracks, Error> {
        self.oauth.update().await?;

        let mut query = Vec::new();
        if let Some(market) = self.market {
            query.push(("market", market));
        }
        if let Some(limit) = self.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            query.push(("offset", offset.to_string()));
        }

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetSavedTracksIter<'a> {
    oauth: &'a mut OAuth,
    market: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl<'a> GetSavedTracksIter<'a> {
    pub fn new(builder: GetSavedTracksBuilder<'a>) -> Self {
        Self {
            oauth: builder.oauth,
            market: builder.market,
            limit: builder.limit,
            offset: builder.offset,
        }
    }
}

impl<'a> AsyncIter for GetSavedTracksIter<'a> {
    type Item = Result<PagedTracks, Error>;

    async fn prev(&mut self) -> Option<Self::Item> {
        if let Err(err) = self.oauth.update().await {
            return Some(Err(Error::from(err)));
        };

        let mut query = Vec::new();
        if let Some(market) = self.market.clone() {
            query.push(("market", market));
        }
        if let Some(limit) = self.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            query.push(("offset", offset.to_string()));
        }

        let result: Result<PagedTracks, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await;

        match result {
            Ok(tracks) => {
                if let Some(prev) = &tracks.previous {
                    self.offset = prev.offset;
                    self.limit = prev.limit;
                }
                Some(Ok(tracks))
            }
            Err(err) => Some(Err(err)),
        }
    }

    async fn next(&mut self) -> Option<Self::Item> {
        if let Err(err) = self.oauth.update().await {
            return Some(Err(Error::from(err)));
        };

        let mut query = Vec::new();
        if let Some(market) = self.market.clone() {
            query.push(("market", market));
        }
        if let Some(limit) = self.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            query.push(("offset", offset.to_string()));
        }

        let result: Result<PagedTracks, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await;

        match result {
            Ok(tracks) => {
                if let Some(next) = &tracks.next {
                    self.offset = next.offset;
                    self.limit = next.limit;
                }
                Some(Ok(tracks))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

/// Check saved tracks
pub struct CheckSavedTracksBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
}

impl<'a> CheckSavedTracksBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl<'a> SpotifyRequest<Vec<bool>> for CheckSavedTracksBuilder<'a> {
    async fn send(mut self) -> Result<Vec<bool>, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/tracks/contains")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(&[("ids", self.ids.join(","))])
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct SaveTracksBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
}

impl<'a> SaveTracksBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl<'a> SpotifyRequest<()> for SaveTracksBuilder<'a> {
    async fn send(mut self) -> Result<(), Error> {
        self.oauth.update().await?;

        let data = &json!({
            "ids": self.ids
        });
        let body = serde_json::to_string(&data)?;

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct RemoveTracksBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
}

impl<'a> RemoveTracksBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl<'a> SpotifyRequest<()> for RemoveTracksBuilder<'a> {
    async fn send(mut self) -> Result<(), Error> {
        self.oauth.update().await?;

        let data = &json!({
            "ids": self.ids
        });

        let body = serde_json::to_string(&data)?;

        reqwest::Client::new()
            .delete("https://api.spotify.com/v1/me/tracks")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}
