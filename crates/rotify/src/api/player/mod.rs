#[cfg(feature = "user-read-playback-state")]
use device::DevicesBuilder;
#[cfg(feature = "user-modify-playback-state")]
use modify::PausePlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use modify::StartPlaybackBuilder;
pub use playback::AdditionalTypes;
#[cfg(feature = "user-read-playback-state")]
use playback::PlayerStateBuilder;
#[cfg(feature = "user-modify-playback-state")]
use playback::TransferPlaybackBuilder;

#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::modify::NextPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::modify::PreviousPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::modify::RepeatBuilder;
#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::modify::SeekPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::modify::ShuffleBuilder;
#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::modify::VolumeBuilder;
#[cfg(feature = "user-modify-playback-state")]
use crate::api::player::queue::AddToQueueBuilder;
#[cfg(all(feature = "user-read-playback-state", feature = "user-read-currently-playing"))]
use crate::api::player::queue::QueueBuilder;
#[cfg(feature = "user-read-recently-played")]
use crate::api::player::queue::RecentlyPlayedBuilder;
use crate::auth::OAuth;
use crate::model::{Uri, UriType};

mod playback;
pub mod device;
mod modify;
mod queue;

pub struct PlayerBuilder<'a>(&'a mut OAuth);

impl<'a> PlayerBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self(oauth)
    }

    /// Get playback state
    ///
    /// # Scope
    /// user-read-playback-state
    #[cfg(feature = "user-read-playback-state")]
    pub fn playback(self) -> PlayerStateBuilder<'a> {
        PlayerStateBuilder::new(self.0)
    }

    /// Get a list of available devices
    #[cfg(feature = "user-read-playback-state")]
    pub fn devices(self) -> DevicesBuilder<'a> {
        DevicesBuilder::new(self.0)
    }

    /// Transfer playback to another device
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn transfer_playback(self) -> TransferPlaybackBuilder<'a> {
        TransferPlaybackBuilder::new(self.0)
    }

    /// Start/Resume playback
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn play(self) -> StartPlaybackBuilder<'a> {
        StartPlaybackBuilder::new(self.0)
    }

    /// Pause playback
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn pause(self) -> PausePlaybackBuilder<'a> {
        PausePlaybackBuilder::new(self.0)
    }

    /// Next item in queue
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn next(self) -> NextPlaybackBuilder<'a> {
        NextPlaybackBuilder::new(self.0)
    }

    /// Previous item in queue
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn previous(self) -> PreviousPlaybackBuilder<'a> {
        PreviousPlaybackBuilder::new(self.0)
    }

    /// Seek forward or backward through the currently playing item
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn seek(self, position: i64) -> SeekPlaybackBuilder<'a> {
        SeekPlaybackBuilder::new(self.0, position)
    }

    // Set playback volume
    #[cfg(feature = "user-modify-playback-state")]
    pub fn volume(self, volume: u8) -> VolumeBuilder<'a> {
        VolumeBuilder::new(self.0, volume)
    }

    /// Toggle shuffle
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn shuffle(self) -> ShuffleBuilder<'a> {
        ShuffleBuilder::new(self.0)
    }

    /// Toggle repeat mode
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn repeat(self) -> RepeatBuilder<'a> {
        RepeatBuilder::new(self.0)
    }

    /// Get recently played tracks
    ///
    /// # Scope
    /// user-read-recently-played
    #[cfg(feature = "user-read-recently-played")]
    pub fn recently_played_tracks(self) -> RecentlyPlayedBuilder<'a> {
        RecentlyPlayedBuilder::new(self.0)
    }

    // Get the current queue
    ///
    /// # Scope
    /// user-read-playback-state, user-read-currently-playing
    #[cfg(all(feature = "user-read-playback-state", feature = "user-read-currently-playing"))]
    pub fn queue(self) -> QueueBuilder<'a> {
        QueueBuilder::new(self.0)
    }

    // Add item to queue
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn add_to_queue<S: Into<String>>(self, uri_type: UriType, id: S) -> AddToQueueBuilder<'a> {
        AddToQueueBuilder::new(self.0, Uri::new(uri_type, id.into()))
    }
}
