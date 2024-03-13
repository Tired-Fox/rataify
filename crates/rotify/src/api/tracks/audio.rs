use std::sync::{Arc, Mutex};
use crate::auth::OAuth;
use crate::model::tracks::{AudioAnalysis, AudioFeatures};
use crate::{Error, SpotifyRequest, SpotifyResponse};

pub struct GetTrackAudioFeaturesBuilder {
    oauth: Arc<Mutex<OAuth>>,
    id: String,
}

impl GetTrackAudioFeaturesBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, id: String) -> Self {
        Self {
            oauth,
            id,
        }
    }
}

impl SpotifyRequest for GetTrackAudioFeaturesBuilder {
    type Response = AudioFeatures;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/audio-features/{}", self.id))
            .header("Authorization", auth.unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetSeveralTrackAudioFeaturesBuilder {
    oauth: Arc<Mutex<OAuth>>,
    ids: Vec<String>,
}

impl GetSeveralTrackAudioFeaturesBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl SpotifyRequest for GetSeveralTrackAudioFeaturesBuilder {
    type Response = Vec<AudioFeatures>;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/audio-features")
            .header("Authorization", auth.unwrap().to_header())
            .query(&[("ids", self.ids.join(","))])
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetTrackAudioAnalysisBuilder {
    oauth: Arc<Mutex<OAuth>>,
    id: String,
}

impl GetTrackAudioAnalysisBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>, id: String) -> Self {
        Self {
            oauth,
            id,
        }
    }
}

impl SpotifyRequest for GetTrackAudioAnalysisBuilder {
    type Response = AudioAnalysis;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/audio-analysis/{}", self.id))
            .header("Authorization", auth.unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}