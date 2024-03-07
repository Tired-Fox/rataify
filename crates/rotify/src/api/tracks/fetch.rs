use crate::{auth::OAuth, Error, SpotifyRequest, SpotifyResponse};
use crate::model::playback::{Track, Tracks};
use crate::model::tracks::PagedTracks;

pub struct GetTrackBuilder<'a> {
    oauth: &'a mut OAuth,
    id: String,
    market: Option<String>,
}

impl<'a> GetTrackBuilder<'a> {
    pub fn new<S: Into<String>>(oauth: &'a mut OAuth, id: S) -> Self {
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

impl<'a> SpotifyRequest<Track> for GetTrackBuilder<'a> {
    async fn send(mut self) -> Result<Track, Error> {
        self.oauth.update().await?;

        let mut query = Vec::new();
        if let Some(market) = self.market {
            query.push(("market", market));
        }

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/tracks/{}", self.id))
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}
pub struct GetTracksBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
    market: Option<String>,
}

impl<'a> GetTracksBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, ids: Vec<String>) -> Self {
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

impl<'a> SpotifyRequest<Tracks> for GetTracksBuilder<'a> {
    async fn send(mut self) -> Result<Tracks, Error> {
        if self.ids.len() > 100 {
            return Err(Error::Unknown("Can fetch a maximum of 100 tracks at a time".into()));
        }

        self.oauth.update().await?;

        let mut query = vec![("ids", self.ids.join(","))];
        if let Some(market) = self.market {
            query.push(("market", market));
        }

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/tracks")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetSavedTracksBuilder<'a> {
    oauth: &'a mut OAuth,
    limit: Option<u8>,
    offset: Option<u8>,
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

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u8) -> Self {
        self.offset = Some(offset);
        self
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
