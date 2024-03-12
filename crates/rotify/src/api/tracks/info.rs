use std::collections::HashMap;
use crate::{auth::OAuth, Error, SpotifyRequest, SpotifyResponse};
use crate::model::audio::{AudioAnalysis, AudioFeatures};
use crate::model::playback::{Track, Tracks};
use crate::model::tracks::Recommendations;

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

impl<'a> SpotifyRequest for GetTrackBuilder<'a> {
    type Response = Track;

    async fn send(mut self) -> Result<Self::Response, Error> {
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

impl<'a> SpotifyRequest for GetTracksBuilder<'a> {
    type Response = Tracks;

    async fn send(mut self) -> Result<Self::Response, Error> {
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

pub struct GetTrackAudioFeaturesBuilder<'a> {
    oauth: &'a mut OAuth,
    id: String,
}

impl<'a> GetTrackAudioFeaturesBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, id: String) -> Self {
        Self {
            oauth,
            id,
        }
    }
}

impl<'a> SpotifyRequest for GetTrackAudioFeaturesBuilder<'a> {
    type Response = AudioFeatures;

    async fn send(mut self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/audio-features/{}", self.id))
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetSeveralTrackAudioFeaturesBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
}

impl<'a> GetSeveralTrackAudioFeaturesBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
        }
    }
}

impl<'a> SpotifyRequest for GetSeveralTrackAudioFeaturesBuilder<'a> {
    type Response = Vec<AudioFeatures>;

    async fn send(mut self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/audio-features")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(&[("ids", self.ids.join(","))])
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetTrackAudioAnalysisBuilder<'a> {
    oauth: &'a mut OAuth,
    id: String,
}

impl<'a> GetTrackAudioAnalysisBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, id: String) -> Self {
        Self {
            oauth,
            id,
        }
    }
}

impl<'a> SpotifyRequest for GetTrackAudioAnalysisBuilder<'a> {
    type Response = AudioAnalysis;

    async fn send(mut self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/audio-analysis/{}", self.id))
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetRecommendationsBuilderSetup<'a> {
    oauth: &'a mut OAuth,
}

impl<'a> GetRecommendationsBuilderSetup<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
        }
    }

    pub fn seed_artists<D: Into<String>, S: IntoIterator<Item = D>>(mut self, seed_artists: S) -> GetRecommendationsBuilder<'a> {
        GetRecommendationsBuilder {
            oauth: self.oauth,
            fields: HashMap::from([
                ("seed_artists", seed_artists.into_iter().map(Into::into).collect::<Vec<String>>().join(",")),
            ]),
        }
    }

    pub fn seed_genres<D: Into<String>, S: IntoIterator<Item = D>>(mut self, seed_genres: S) -> GetRecommendationsBuilder<'a> {
        GetRecommendationsBuilder {
            oauth: self.oauth,
            fields: HashMap::from([
                ("seed_genres", seed_genres.into_iter().map(Into::into).collect::<Vec<String>>().join(",")),
            ]),
        }
    }

    pub fn seed_tracks<D: Into<String>, S: IntoIterator<Item = D>>(mut self, seed_tracks: S) -> GetRecommendationsBuilder<'a> {
        GetRecommendationsBuilder {
            oauth: self.oauth,
            fields: HashMap::from([
                ("seed_tracks", seed_tracks.into_iter().map(Into::into).collect::<Vec<String>>().join(",")),
            ]),
        }
    }
}

pub struct GetRecommendationsBuilder<'a> {
    oauth: &'a mut OAuth,
    fields: HashMap<&'static str, String>,
}

impl<'a> GetRecommendationsBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            fields: HashMap::new(),
        }
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.fields.insert("market", market.into());
        self
    }

    pub fn seed_artists<D: Into<String>, S: IntoIterator<Item = D>>(mut self, seed_artists: S) -> GetRecommendationsBuilder<'a> {
        self.fields.insert("seed_artists", seed_artists.into_iter().map(Into::into).collect::<Vec<String>>().join(","));
        self
    }

    pub fn seed_genres<D: Into<String>, S: IntoIterator<Item = D>>(mut self, seed_genres: S) -> GetRecommendationsBuilder<'a> {
        self.fields.insert("seed_genres", seed_genres.into_iter().map(Into::into).collect::<Vec<String>>().join(","));
        self
    }

    pub fn seed_tracks<D: Into<String>, S: IntoIterator<Item = D>>(mut self, seed_tracks: S) -> GetRecommendationsBuilder<'a> {
        self.fields.insert("seed_tracks", seed_tracks.into_iter().map(Into::into).collect::<Vec<String>>().join(","));
        self
    }

    pub fn min_acousticness(mut self, min_acousticness: f32) -> Self {
        self.fields.insert("min_acousticness", min_acousticness.to_string());
        self
    }

    pub fn max_acousticness(mut self, max_acousticness: f32) -> Self {
        self.fields.insert("max_acousticness", max_acousticness.to_string());
        self
    }

    pub fn target_acousticness(mut self, target_acousticness: f32) -> Self {
        self.fields.insert("target_acousticness", target_acousticness.to_string());
        self
    }

    pub fn min_danceability(mut self, min_danceability: f32) -> Self {
        self.fields.insert("min_danceability", min_danceability.to_string());
        self
    }

    pub fn max_danceability(mut self, max_danceability: f32) -> Self {
        self.fields.insert("max_danceability", max_danceability.to_string());
        self
    }

    pub fn target_danceability(mut self, target_danceability: f32) -> Self {
        self.fields.insert("target_danceability", target_danceability.to_string());
        self
    }

    pub fn min_duration_ms(mut self, min_duration_ms: u32) -> Self {
        self.fields.insert("min_duration_ms", min_duration_ms.to_string());
        self
    }

    pub fn max_duration_ms(mut self, max_duration_ms: u32) -> Self {
        self.fields.insert("max_duration_ms", max_duration_ms.to_string());
        self
    }

    pub fn target_duration_ms(mut self, target_duration_ms: u32) -> Self {
        self.fields.insert("target_duration_ms", target_duration_ms.to_string());
        self
    }

    pub fn min_energy(mut self, min_energy: f32) -> Self {
        self.fields.insert("min_energy", min_energy.to_string());
        self
    }

    pub fn max_energy(mut self, max_energy: f32) -> Self {
        self.fields.insert("max_energy", max_energy.to_string());
        self
    }

    pub fn target_energy(mut self, target_energy: f32) -> Self {
        self.fields.insert("target_energy", target_energy.to_string());
        self
    }

    pub fn min_instrumentalness(mut self, min_instrumentalness: f32) -> Self {
        self.fields.insert("min_instrumentalness", min_instrumentalness.to_string());
        self
    }

    pub fn max_instrumentalness(mut self, max_instrumentalness: f32) -> Self {
        self.fields.insert("max_instrumentalness", max_instrumentalness.to_string());
        self
    }

    pub fn target_instrumentalness(mut self, target_instrumentalness: f32) -> Self {
        self.fields.insert("target_instrumentalness", target_instrumentalness.to_string());
        self
    }

    pub fn min_key(mut self, min_key: u8) -> Self {
        self.fields.insert("min_key", min_key.to_string());
        self
    }

    pub fn max_key(mut self, max_key: u8) -> Self {
        self.fields.insert("max_key", max_key.to_string());
        self
    }

    pub fn target_key(mut self, target_key: u8) -> Self {
        self.fields.insert("target_key", target_key.to_string());
        self
    }

    pub fn min_liveness(mut self, min_liveness: f32) -> Self {
        self.fields.insert("min_liveness", min_liveness.to_string());
        self
    }

    pub fn max_liveness(mut self, max_liveness: f32) -> Self {
        self.fields.insert("max_liveness", max_liveness.to_string());
        self
    }

    pub fn target_liveness(mut self, target_liveness: f32) -> Self {
        self.fields.insert("target_liveness", target_liveness.to_string());
        self
    }

    pub fn min_loudness(mut self, min_loudness: f32) -> Self {
        self.fields.insert("min_loudness", min_loudness.to_string());
        self
    }

    pub fn max_loudness(mut self, max_loudness: f32) -> Self {
        self.fields.insert("max_loudness", max_loudness.to_string());
        self
    }

    pub fn target_loudness(mut self, target_loudness: f32) -> Self {
        self.fields.insert("target_loudness", target_loudness.to_string());
        self
    }

    pub fn min_popularity(mut self, min_popularity: u8) -> Self {
        self.fields.insert("min_popularity", min_popularity.to_string());
        self
    }

    pub fn max_popularity(mut self, max_popularity: u8) -> Self {
        self.fields.insert("max_popularity", max_popularity.to_string());
        self
    }

    pub fn target_popularity(mut self, target_popularity: u8) -> Self {
        self.fields.insert("target_popularity", target_popularity.to_string());
        self
    }

    pub fn min_tempo(mut self, min_tempo: f32) -> Self {
        self.fields.insert("min_tempo", min_tempo.to_string());
        self
    }

    pub fn max_tempo(mut self, max_tempo: f32) -> Self {
        self.fields.insert("max_tempo", max_tempo.to_string());
        self
    }

    pub fn target_tempo(mut self, target_tempo: f32) -> Self {
        self.fields.insert("target_tempo", target_tempo.to_string());
        self
    }

    pub fn min_mod(mut self, min_mod: u8) -> Self {
        self.fields.insert("min_mod", min_mod.to_string());
        self
    }

    pub fn max_mod(mut self, max_mod: u8) -> Self {
        self.fields.insert("max_mod", max_mod.to_string());
        self
    }

    pub fn target_mod(mut self, target_mod: u8) -> Self {
        self.fields.insert("target_mod", target_mod.to_string());
        self
    }

    pub fn min_speechiness(mut self, min_speechiness: f32) -> Self {
        self.fields.insert("min_speechiness", min_speechiness.to_string());
        self
    }

    pub fn max_speechiness(mut self, max_speechiness: f32) -> Self {
        self.fields.insert("max_speechiness", max_speechiness.to_string());
        self
    }

    pub fn target_speechiness(mut self, target_speechiness: f32) -> Self {
        self.fields.insert("target_speechiness", target_speechiness.to_string());
        self
    }

    pub fn min_time_signature(mut self, min_time_signature: u8) -> Self {
        self.fields.insert("min_time_signature", min_time_signature.to_string());
        self
    }

    pub fn max_time_signature(mut self, max_time_signature: u8) -> Self {
        self.fields.insert("max_time_signature", max_time_signature.to_string());
        self
    }

    pub fn target_time_signature(mut self, target_time_signature: u8) -> Self {
        self.fields.insert("target_time_signature", target_time_signature.to_string());
        self
    }

    pub fn min_valence(mut self, min_valence: f32) -> Self {
        self.fields.insert("min_valence", min_valence.to_string());
        self
    }

    pub fn max_valence(mut self, max_valence: f32) -> Self {
        self.fields.insert("max_valence", max_valence.to_string());
        self
    }

    pub fn target_valence(mut self, target_valence: f32) -> Self {
        self.fields.insert("target_valence", target_valence.to_string());
        self
    }
}

impl<'a> SpotifyRequest for GetRecommendationsBuilder<'a> {
    type Response = Recommendations;

    async fn send(mut self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/recommendations")
            .header("Authorization", self.oauth.token.as_ref().unwrap().to_header())
            .query(&self.fields)
            .send()
            .to_spotify_response()
            .await
    }
}
