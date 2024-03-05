use serde_json::{json, Map, Value};

use crate::spot::{auth::OAuth, Error, NoContent, SpotifyRequest, SpotifyResponse};

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

    pub fn devices<S: IntoIterator<Item = String>>(mut self, devices: S) -> Self {
        self.devices = devices.into_iter().collect::<Vec<String>>();
        self
    }

    pub fn play(mut self, play: bool) -> Self {
        self.play = Some(play);
        self
    }
}

impl<'a> SpotifyRequest<NoContent> for TransferPlaybackBuilder<'a> {
    async fn send(self) -> Result<NoContent, crate::spot::Error> {
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
            .header(
                "Authorization",
                self.oauth.token.as_ref().unwrap().to_header(),
            )
            .body(serde_json::to_string(&body).map_err(|e| Error::Unknown(e.to_string()))?)
            .send()
            .await
            .to_spotify_response()
            .await
    }
}
