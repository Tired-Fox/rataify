use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{auth::OAuth, Error, SpotifyRequest, SpotifyResponse};
use crate::model::playback::Playback;

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AdditionalTypes {
    #[default]
    Track,
    Episode,
}

pub struct PlayerStateBuilder<'a> {
    oauth: &'a mut OAuth,
    device_id: Option<String>,
    market: Option<String>,
    additional_types: Vec<AdditionalTypes>,
}

impl<'a> PlayerStateBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
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

impl<'a> SpotifyRequest for PlayerStateBuilder<'a> {
    type Response = Option<Playback>;

    async fn send(mut self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
        let result: Result<Playback, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/player")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
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

pub struct TransferPlaybackBuilder<'a> {
    oauth: &'a mut OAuth,
    devices: Vec<String>,
    play: Option<bool>,
}

impl<'a> TransferPlaybackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            devices: Vec::new(),
            play: None,
        }
    }

    pub fn device<S: Into<String>>(mut self, device: S) -> Self {
        self.devices.push(device.into());
        self
    }

    pub fn devices<S: IntoIterator<Item=String>>(mut self, devices: S) -> Self {
        self.devices = devices.into_iter().collect::<Vec<String>>();
        self
    }

    pub fn play(mut self, play: bool) -> Self {
        self.play = Some(play);
        self
    }
}

impl<'a> SpotifyRequest for TransferPlaybackBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;
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
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .body(serde_json::to_string(&body).map_err(|e| Error::Unknown(e.to_string()))?)
            .send()
            .to_spotify_response()
            .await
    }
}
