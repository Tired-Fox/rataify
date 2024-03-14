use crate::{auth::OAuth, SpotifyResponse};
use serde::Deserialize;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crate::model::Wrapped;
use crate::model::player::{Device};

use crate::SpotifyRequest;

pub struct DevicesBuilder {
    oauth: Arc<Mutex<OAuth>>,
}

impl DevicesBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self { oauth }
    }
}

impl SpotifyRequest for DevicesBuilder {
    type Response = Vec<Device>;

    async fn send(self) -> Result<Self::Response, crate::Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        let result: Result<Wrapped<Vec<Device>>, crate::Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization", auth.unwrap().to_header())
            .send()
            .to_spotify_response()
            .await;

        result.map(|v| v.unwrap())
    }
}
