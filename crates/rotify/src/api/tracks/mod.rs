use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use audio::{GetSeveralTrackAudioFeaturesBuilder, GetTrackAudioAnalysisBuilder, GetTrackAudioFeaturesBuilder};
#[cfg(feature = "user-library-read")]
use track::{CheckSavedTracksBuilder, GetSavedTracksBuilder};
#[cfg(feature = "user-library-modify")]
use track::{RemoveTracksBuilder, SaveTracksBuilder};
use track::{GetTrackBuilder, GetTracksBuilder};

use crate::{AsyncIter, Error, SpotifyRequest, SpotifyResponse};
use crate::auth::OAuth;
use crate::model::tracks::Recommendations;

mod track;
mod audio;

pub struct TrackBuilder(Arc<Mutex<OAuth>>);

impl TrackBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self(oauth)
    }

    /// Get catalog information for a single track
    pub fn get_track<S: Into<String>>(self, id: S) -> GetTrackBuilder {
        GetTrackBuilder::new(self.0, id)
    }

    /// Get catalog information for multiple tracks
    pub fn get_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> GetTracksBuilder {
        GetTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Get list of the current user's saved tracks
    ///
    /// # Scope
    /// user-library-read
    #[cfg(feature = "user-library-read")]
    pub fn get_saved_tracks(self) -> GetSavedTracksBuilder {
        GetSavedTracksBuilder::new(self.0)
    }

    /// Save tracks for current user
    ///
    /// # Scope
    /// user-library-modify
    #[cfg(feature = "user-library-modify")]
    pub fn save_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> SaveTracksBuilder {
        SaveTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Remove tracks from current user's saved tracks
    ///
    /// # Scope
    /// user-library-modify
    #[cfg(feature = "user-library-modify")]
    pub fn remove_saved_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> RemoveTracksBuilder {
        RemoveTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Check if one or more tracks is already saved by the current user
    ///
    /// # Scope
    /// user-library-read
    #[cfg(feature = "user-library-read")]
    pub fn check_saved_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> CheckSavedTracksBuilder {
        CheckSavedTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Get audio features for a single track
    pub fn get_track_audio_features<S: Into<String>>(self, id: S) -> GetTrackAudioFeaturesBuilder {
        GetTrackAudioFeaturesBuilder::new(self.0, id.into())
    }

    /// Get audio features for a several tracks
    pub fn get_several_tracks_audio_features<S: IntoIterator<Item=String>>(self, ids: S) -> GetSeveralTrackAudioFeaturesBuilder {
        GetSeveralTrackAudioFeaturesBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Get audio analysis for a single track
    pub fn get_track_audio_analysis<S: Into<String>>(self, id: S) -> GetTrackAudioAnalysisBuilder {
        GetTrackAudioAnalysisBuilder::new(self.0, id.into())
    }

    /// Get recommendations based on the seed information
    pub fn get_recommendations(self) -> GetRecommendationsBuilderSetup {
        GetRecommendationsBuilderSetup::new(self.0)
    }
}

pub struct GetRecommendationsBuilderSetup {
    oauth: Arc<Mutex<OAuth>>,
}

impl GetRecommendationsBuilderSetup {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self {
            oauth,
        }
    }

    pub fn seed_artists<D: Into<String>, S: IntoIterator<Item=D>>(mut self, seed_artists: S) -> GetRecommendationsBuilder {
        GetRecommendationsBuilder {
            oauth: self.oauth,
            fields: HashMap::from([
                ("seed_artists", seed_artists.into_iter().map(Into::into).collect::<Vec<String>>().join(",")),
            ]),
        }
    }

    pub fn seed_genres<D: Into<String>, S: IntoIterator<Item=D>>(mut self, seed_genres: S) -> GetRecommendationsBuilder {
        GetRecommendationsBuilder {
            oauth: self.oauth,
            fields: HashMap::from([
                ("seed_genres", seed_genres.into_iter().map(Into::into).collect::<Vec<String>>().join(",")),
            ]),
        }
    }

    pub fn seed_tracks<D: Into<String>, S: IntoIterator<Item=D>>(mut self, seed_tracks: S) -> GetRecommendationsBuilder {
        GetRecommendationsBuilder {
            oauth: self.oauth,
            fields: HashMap::from([
                ("seed_tracks", seed_tracks.into_iter().map(Into::into).collect::<Vec<String>>().join(",")),
            ]),
        }
    }
}

pub struct GetRecommendationsBuilder {
    oauth: Arc<Mutex<OAuth>>,
    fields: HashMap<&'static str, String>,
}

impl GetRecommendationsBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self {
            oauth,
            fields: HashMap::new(),
        }
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.fields.insert("market", market.into());
        self
    }

    pub fn seed_artists<D: Into<String>, S: IntoIterator<Item=D>>(mut self, seed_artists: S) -> GetRecommendationsBuilder {
        self.fields.insert("seed_artists", seed_artists.into_iter().map(Into::into).collect::<Vec<String>>().join(","));
        self
    }

    pub fn seed_genres<D: Into<String>, S: IntoIterator<Item=D>>(mut self, seed_genres: S) -> GetRecommendationsBuilder {
        self.fields.insert("seed_genres", seed_genres.into_iter().map(Into::into).collect::<Vec<String>>().join(","));
        self
    }

    pub fn seed_tracks<D: Into<String>, S: IntoIterator<Item=D>>(mut self, seed_tracks: S) -> GetRecommendationsBuilder {
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

impl SpotifyRequest for GetRecommendationsBuilder {
    type Response = Recommendations;

    async fn send(mut self) -> Result<Self::Response, Error> {
        let auth = self.oauth.lock().unwrap().update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/recommendations")
            .header("Authorization", auth.unwrap().to_header())
            .query(&self.fields)
            .send()
            .to_spotify_response()
            .await
    }
}