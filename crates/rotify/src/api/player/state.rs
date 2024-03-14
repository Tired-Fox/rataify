use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{auth::OAuth, Error, SpotifyRequest, SpotifyResponse};
use crate::model::player::{Playback, Repeat};
use crate::model::Uri;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AdditionalTypes {
    #[default]
    Track,
    Episode,
}

pub struct PlayerStateBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device_id: Option<String>,
    market: Option<String>,
    additional_types: Vec<AdditionalTypes>,
}

impl PlayerStateBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self {
            oauth,
            device_id: None,
            market: None,
            additional_types: vec![AdditionalTypes::Track],
        }
    }

    pub fn device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.market = Some(market.into());
        self
    }

    pub fn additional_types<const N: usize>(
        mut self,
        additional_types: [AdditionalTypes; N],
    ) -> Self {
        self.additional_types = additional_types.to_vec();
        self
    }
}

impl SpotifyRequest for PlayerStateBuilder {
    type Response = Option<Playback>;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;
        let result: Result<Playback, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/player")
            .header("Authorization", auth.unwrap().to_header())
            .send()
            .to_spotify_response()
            .await;

        // Map no content error to be None. This is because the playback just isn't available at the moment
        match result {
            Ok(result) => Ok(Some(result)),
            Err(Error::NoContent) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub struct TransferPlaybackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    devices: Vec<String>,
    play: Option<bool>,
}

impl TransferPlaybackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, ids: Vec<String>) -> Self {
        Self {
            oauth,
            devices: ids,
            play: None,
        }
    }

    pub fn play(mut self, play: bool) -> Self {
        self.play = Some(play);
        self
    }
}

impl SpotifyRequest for TransferPlaybackBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;
        let mut body = Value::Object(Map::new());
        if let Value::Object(map) = &mut body {
            map.insert(
                "device_ids".to_string(),
                Value::Array(
                    self.devices
                        .iter()
                        .map(|v| Value::String(v.clone()))
                        .collect(),
                ),
            );
            if let Some(play) = self.play {
                map.insert("play".to_string(), Value::Bool(play));
            }
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player")
            .header("Authorization", auth.unwrap().to_header())
            .body(serde_json::to_string(&body).map_err(|e| Error::Unknown(e.to_string()))?)
            .send()
            .to_spotify_response()
            .await
    }
}

/// start
pub struct StartPlaybackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device: Option<String>,
    context: Option<Uri>,
    uris: Vec<Uri>,
    offset: Option<usize>,
    position: usize,
}

impl StartPlaybackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
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

impl SpotifyRequest for StartPlaybackBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth =self.oauth.lock().unwrap().update().await?;
        let mut request = reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/play")
            .header("Authorization", auth.unwrap().to_header());
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
pub struct PausePlaybackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device: Option<String>,
}

impl PausePlaybackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
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

impl SpotifyRequest for PausePlaybackBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth =self.oauth.lock().unwrap().update().await?;
        let mut request = reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/pause")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header());

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
pub struct NextPlaybackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device: Option<String>,
}

impl NextPlaybackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
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

impl SpotifyRequest for NextPlaybackBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;
        reqwest::Client::new()
            .post("https://api.spotify.com/v1/me/player/next")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

/// previous
pub struct PreviousPlaybackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device: Option<String>,
}

impl PreviousPlaybackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
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

impl SpotifyRequest for PreviousPlaybackBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;
        reqwest::Client::new()
            .post("https://api.spotify.com/v1/me/player/previous")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

/// Seek
pub struct SeekPlaybackBuilder {
    oauth: Arc<Mutex<OAuth>>,
    position: i64,
    device: Option<String>,
}

impl SeekPlaybackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, position: i64) -> Self {
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

impl SpotifyRequest for SeekPlaybackBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        let mut query = vec![("position_ms", self.position.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/seek")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

// volume
pub struct VolumeBuilder {
    oauth: Arc<Mutex<OAuth>>,
    volume_percent: u8,
    device: Option<String>,
}

impl VolumeBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, mut volume_percent: u8) -> Self {
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

impl SpotifyRequest for VolumeBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        let mut query = vec![("volume_percent", self.volume_percent.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/volume")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

/// Shuffle
pub struct ShuffleBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device: Option<String>,
    state: bool,
}

impl ShuffleBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, state: bool) -> Self {
        Self {
            oauth,
            state,
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

impl SpotifyRequest for ShuffleBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        let mut query = vec![("state", self.state.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/shuffle")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}

/// Repeat
pub struct RepeatBuilder {
    oauth: Arc<Mutex<OAuth>>,
    device: Option<String>,
    state: Repeat,
}

impl RepeatBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, repeat: Repeat) -> Self {
        Self {
            oauth,
            state: repeat,
            device: None,
        }
    }

    pub fn device<S: Into<String>>(mut self, device: S) -> Self {
        self.device = Some(device.into());
        self
    }
}

impl SpotifyRequest for RepeatBuilder {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;
        let mut query = vec![("state", self.state.to_string())];
        if let Some(device) = self.device {
            query.push(("device_id", device));
        }

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/player/repeat")
            .header("Content-Length", 0)
            .header("Authorization", auth.unwrap().to_header())
            .query(query.as_slice())
            .send()
            .to_spotify_response()
            .await
    }
}
