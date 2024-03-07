use crate::api::tracks::fetch::{GetTrackBuilder, GetTracksBuilder};
#[cfg(feature = "user-library-read")]
use crate::api::tracks::fetch::GetSavedTracksBuilder;
use crate::auth::OAuth;

mod fetch;

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
}
