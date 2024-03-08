#[cfg(feature = "user-library-read")]
use user::CheckSavedTracksBuilder;
#[cfg(feature = "user-library-read")]
use user::GetSavedTracksBuilder;
#[cfg(feature = "user-library-modify")]
use user::SaveTracksBuilder;
#[cfg(feature = "user-library-modify")]
use crate::api::tracks::user::RemoveTracksBuilder;

use crate::api::tracks::info::{GetTrackBuilder, GetTracksBuilder, GetTrackAudioFeaturesBuilder, GetTrackAudioAnalysisBuilder, GetSeveralTrackAudioFeaturesBuilder, GetRecommendationsBuilder, GetRecommendationsBuilderSetup};
use crate::auth::OAuth;

#[cfg(any(feature = "user-library-read", feature = "user-library-modify"))]
mod user;

mod info;
mod audio;

pub struct TrackBuilder<'a>(&'a mut OAuth);

impl<'a> TrackBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self(oauth)
    }

    /// Get catalog information for a single track
    pub fn get_track<S: Into<String>>(self, id: S) -> GetTrackBuilder<'a> {
        GetTrackBuilder::new(self.0, id)
    }

    /// Get catalog information for multiple tracks
    pub fn get_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> GetTracksBuilder<'a> {
        GetTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Get list of the current user's saved tracks
    ///
    /// # Scope
    /// user-library-read
    #[cfg(feature = "user-library-read")]
    pub fn get_saved_tracks(self) -> GetSavedTracksBuilder<'a> {
        GetSavedTracksBuilder::new(self.0)
    }

    /// Check if one or more tracks is already saved by the current user
    ///
    /// # Scope
    /// user-library-read
    #[cfg(feature = "user-library-read")]
    pub fn check_saved_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> CheckSavedTracksBuilder<'a> {
        CheckSavedTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Save tracks for current user
    ///
    /// # Scope
    /// user-library-modify
    #[cfg(feature = "user-library-modify")]
    pub fn save_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> SaveTracksBuilder<'a> {
        SaveTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Remove tracks from current user's saved tracks
    ///
    /// # Scope
    /// user-library-modify
    #[cfg(feature = "user-library-modify")]
    pub fn remove_saved_tracks<S: IntoIterator<Item=String>>(self, ids: S) -> RemoveTracksBuilder<'a> {
        RemoveTracksBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Get audio features for a single track
    pub fn get_track_audio_features<S: Into<String>>(self, id: S) -> GetTrackAudioFeaturesBuilder<'a> {
        GetTrackAudioFeaturesBuilder::new(self.0, id.into())
    }

    /// Get audio features for a several tracks
    pub fn get_several_track_audio_features<S: IntoIterator<Item=String>>(self, ids: S) -> GetSeveralTrackAudioFeaturesBuilder<'a> {
        GetSeveralTrackAudioFeaturesBuilder::new(self.0, ids.into_iter().collect())
    }

    /// Get audio analysis for a single track
    pub fn get_track_audio_analysis<S: Into<String>>(self, id: S) -> GetTrackAudioAnalysisBuilder<'a> {
        GetTrackAudioAnalysisBuilder::new(self.0, id.into())
    }

    /// Get recommendations based on the seed information
    pub fn get_recommendations(self) -> GetRecommendationsBuilderSetup<'a> {
        GetRecommendationsBuilderSetup::new(self.0)
    }

    /*
    TODO: get recommendations
     */
}
