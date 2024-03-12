use chrono::Duration;

use crate::{Error, SpotifyRequest, SpotifyResponse};
use crate::auth::OAuth;
use crate::model::queue::{Queue, RecentlyPlayedTracks};
use crate::model::Uri;

enum TimeOffset {
    Before,
    After,
}

impl TimeOffset {
    fn as_str(&self) -> &str {
        match self {
            TimeOffset::Before => "before",
            TimeOffset::After => "after"
        }
    }
}

pub struct RecentlyPlayedBuilder<'a> {
    oauth: &'a mut OAuth,
    amount: Option<Duration>,
    limit: Option<u8>,
    offset: TimeOffset,
}

impl<'a> RecentlyPlayedBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            amount: None,
            limit: None,
            offset: TimeOffset::Before,
        }
    }

    pub fn before(mut self, amount: Duration) -> Self {
        self.amount = Some(amount);
        self.offset = TimeOffset::Before;
        self
    }

    pub fn after(mut self, amount: Duration) -> Self {
        self.amount = Some(amount);
        self.offset = TimeOffset::After;
        self
    }

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = Some(limit.min(50));
        self
    }
}

impl<'a> SpotifyRequest for RecentlyPlayedBuilder<'a> {
    type Response = RecentlyPlayedTracks;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let mut query = Vec::new();
        if let Some(amount) = self.amount {
            query.push((self.offset.as_str(), amount.num_milliseconds().to_string()));
        }
        if let Some(limit) = self.limit {
            query.push(("limit", limit.to_string()));
        }

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/player/recently-played")
            .query(&query)
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

// Queue
pub struct QueueBuilder<'a> {
    oauth: &'a mut OAuth,
}

impl<'a> QueueBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth
        }
    }
}

impl<'a> SpotifyRequest for QueueBuilder<'a> {
    type Response = Queue;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/player/queue")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

// Add to queue
pub struct AddToQueueBuilder<'a> {
    oauth: &'a mut OAuth,
    uri: Uri,
    device_id: Option<String>,
}

impl<'a> AddToQueueBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, uri: Uri) -> Self {
        Self {
            oauth,
            uri,
            device_id: None,
        }
    }

    pub fn device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }
}

impl<'a> SpotifyRequest for AddToQueueBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let mut query = vec![("uri", self.uri.to_string())];
        if let Some(device_id) = self.device_id {
            query.push(("device_id", device_id));
        }

        reqwest::Client::new()
            .post("https://api.spotify.com/v1/me/player/queue")
            .header("Content-Length", 0)
            .query(&query)
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}