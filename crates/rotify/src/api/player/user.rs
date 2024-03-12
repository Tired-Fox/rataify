use std::fmt::Display;
use std::str::FromStr;
use serde_json::json;
use crate::auth::OAuth;
use crate::{Error, NoContent, SpotifyRequest, SpotifyResponse};
use crate::model::Uri;
use crate::model::playback::Repeat;

/// start
pub struct StartPlaybackBuilder<'a> {
    oauth: &'a mut OAuth,
    device: Option<String>,
    context: Option<Uri>,
    uris: Vec<Uri>,
    offset: Option<usize>,
    position: usize,
}

impl<'a> StartPlaybackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            device: None,
            context: None,
            uris: Vec::new(),
            offset: None,
            position: 0,
        }
    }

    pub fn device(mut self, device: String) -> Self {
        self.device = Some(device);
        self
    }

    pub fn context(mut self, context: &str) -> Self {
        self.context = Some(Uri::from_str(context).unwrap());
        self
    }

    pub fn uris(mut self, uris: Vec<Uri>) -> Self {
        self.uris = uris;
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uris.push(Uri::from_str(uri).unwrap());
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }
}

impl<'a> SpotifyRequest for StartPlaybackBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
        let mut request = reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/play")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header());
        if let Some(device) = self.device {
            request = request.query(&[("device_id", device)]);
        }

        let mut body = json!({
            "position_ms": self.position,
        });

        if let Some(context) = self.context {
            body["context_uri"] = json!(context);
        }

        if let Some(offset) = self.offset {
            body["offset"] = json!({
                "position": offset
            })
        }

        if !self.uris.is_empty() {
            body["uris"] = json!(self.uris);
        }

        let body = serde_json::to_string(&body)?;

        request
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

/// pause
pub struct PausePlaybackBuilder<'a> {
    oauth: &'a mut OAuth,
    device: Option<String>,
}

impl<'a> PausePlaybackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            device: None,
        }
    }

    pub fn device(mut self, device: String) -> Self {
        self.device = Some(device);
        self
    }
}

impl<'a> SpotifyRequest for PausePlaybackBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
        let mut request = reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/pause")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header());

        if let Some(device) = self.device {
            request = request.query(&[("device_id", device)]);
        }

        request
            .send()
            .to_spotify_response()
            .await
    }
}

/// next
pub struct NextPlaybackBuilder<'a> {
    oauth: &'a mut OAuth,
    device: Option<String>,
}

impl<'a> NextPlaybackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            device: None,
        }
    }

    pub fn device(mut self, device: String) -> Self {
        self.device = Some(device);
        self
    }
}

impl<'a> SpotifyRequest for NextPlaybackBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
        reqwest::Client::new()
            .post("https://api.spotify.com/v1/me/player/next")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

/// previous
pub struct PreviousPlaybackBuilder<'a> {
    oauth: &'a mut OAuth,
    device: Option<String>,
}

impl<'a> PreviousPlaybackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            device: None,
        }
    }

    pub fn device(mut self, device: String) -> Self {
        self.device = Some(device);
        self
    }
}

impl<'a> SpotifyRequest for PreviousPlaybackBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
        reqwest::Client::new()
            .post("https://api.spotify.com/v1/me/player/previous")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

/// Seek
pub struct SeekPlaybackBuilder<'a> {
    oauth: &'a mut OAuth,
    position: i64,
    device: Option<String>,
}

impl<'a> SeekPlaybackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, position: i64) -> Self {
        Self {
            oauth,
            device: None,
            position,
        }
    }

    pub fn device(mut self, device: String) -> Self {
        self.device = Some(device);
        self
    }
}

impl<'a> SpotifyRequest for SeekPlaybackBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let mut query = vec![("position_ms", self.position.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/seek")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

// volume
pub struct VolumeBuilder<'a> {
    oauth: &'a mut OAuth,
    volume_percent: u8,
    device: Option<String>,
}

impl<'a> VolumeBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, mut volume_percent: u8) -> Self {
        if volume_percent > 100 {
            volume_percent = 100;
        }

        Self {
            oauth,
            device: None,
            volume_percent,
        }
    }

    pub fn device(mut self, device: String) -> Self {
        self.device = Some(device);
        self
    }
}

impl<'a> SpotifyRequest for VolumeBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let mut query = vec![("volume_percent", self.volume_percent.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/volume")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

/// Shuffle
pub struct ShuffleBuilder<'a> {
    oauth: &'a mut OAuth,
    device: Option<String>,
    state: bool,
}

impl<'a> ShuffleBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            state: false,
            device: None,
        }
    }

    pub fn state(mut self, state: bool) -> Self {
        self.state = state;
        self
    }

    pub fn on(self) -> Self {
        self.state(true)
    }

    pub fn off(self) -> Self {
        self.state(false)
    }
}

impl<'a> SpotifyRequest for ShuffleBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let mut query = vec![("state", self.state.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/shuffle")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

/// Repeat
pub struct RepeatBuilder<'a> {
    oauth: &'a mut OAuth,
    device: Option<String>,
    state: Repeat,
}

impl<'a> RepeatBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            state: Repeat::Off,
            device: None,
        }
    }

    pub fn state(mut self, state: Repeat) -> Self {
        self.state = state;
        self
    }

    pub fn track(self) -> Self {
        self.state(Repeat::Track)
    }

    pub fn context(self) -> Self {
        self.state(Repeat::Context)
    }

    pub fn off(self) -> Self {
        self.state(Repeat::Off)
    }
}

impl<'a> SpotifyRequest for RepeatBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
        let mut query = vec![("state", self.state.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/repeat")
            .header("Content-Length", 0)
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}