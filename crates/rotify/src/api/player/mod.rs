use std::sync::{Arc, Mutex};
#[cfg(feature = "user-read-playback-state")]
use device::DevicesBuilder;
#[cfg(feature = "user-modify-playback-state")]
use queue::AddToQueueBuilder;
#[cfg(all(feature = "user-read-playback-state", feature = "user-read-currently-playing"))]
use queue::QueueBuilder;
#[cfg(feature = "user-read-recently-played")]
use queue::RecentlyPlayedBuilder;
pub use state::AdditionalTypes;
#[cfg(feature = "user-modify-playback-state")]
use state::NextPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::PausePlaybackBuilder;
#[cfg(feature = "user-read-playback-state")]
use state::PlayerStateBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::PreviousPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::RepeatBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::SeekPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::ShuffleBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::StartPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::TransferPlaybackBuilder;
#[cfg(feature = "user-modify-playback-state")]
use state::VolumeBuilder;

use crate::auth::OAuth;
use crate::model::{Uri, UriType};
use crate::model::player::Repeat;

mod state;
#[cfg(feature = "user-read-playback-state")]
pub mod device;
#[cfg(any(feature = "user-modify-playback-state", feature = "user-read-playback-state", feature = "user-read-recently-played"))]
mod queue;

pub struct PlayerBuilder(Arc<Mutex<OAuth>>);

impl PlayerBuilder {
    pub fn new(oauth: Arc<Mutex<OAuth>>) -> Self {
        Self(oauth)
    }

    // TODO: currently playing track

    /// Get playback state
    ///
    /// # Scope
    /// user-read-playback-state
    #[cfg(feature = "user-read-playback-state")]
    pub fn playback(self) -> PlayerStateBuilder {
        PlayerStateBuilder::new(self.0)
    }

    /// Transfer playback to another device
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn transfer_playback<T: Into<String>, S: IntoIterator<Item = T>>(self, devices: S) -> TransferPlaybackBuilder {
        TransferPlaybackBuilder::new(self.0, devices.into_iter().map(|v| v.into()).collect())
    }

    /// Get a list of available devices
    #[cfg(feature = "user-read-playback-state")]
    pub fn get_devices(self) -> DevicesBuilder {
        DevicesBuilder::new(self.0)
    }

    /// Start/Resume playback
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn play(self) -> StartPlaybackBuilder {
        StartPlaybackBuilder::new(self.0)
    }

    /// Pause playback
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn pause(self) -> PausePlaybackBuilder {
        PausePlaybackBuilder::new(self.0)
    }

    /// Next item in queue
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn skip_to_next(self) -> NextPlaybackBuilder {
        NextPlaybackBuilder::new(self.0)
    }

    /// Previous item in queue
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn skip_to_previous(self) -> PreviousPlaybackBuilder {
        PreviousPlaybackBuilder::new(self.0)
    }

    /// Seek forward or backward through the currently playing item
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn seek_to_position(self, position: i64) -> SeekPlaybackBuilder {
        SeekPlaybackBuilder::new(self.0, position)
    }

    /// Set playback volume as a percentage
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn set_playback_volume(self, volume: u8) -> VolumeBuilder {
        VolumeBuilder::new(self.0, volume)
    }

    /// Toggle shuffle
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn shuffle(self, shuffle: bool) -> ShuffleBuilder {
        ShuffleBuilder::new(self.0, shuffle)
    }

    /// Toggle repeat mode
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn repeat(self, repeat: Repeat) -> RepeatBuilder {
        RepeatBuilder::new(self.0, repeat)
    }

    /// Get recently played tracks
    ///
    /// # Scope
    /// user-read-recently-played
    #[cfg(feature = "user-read-recently-played")]
    pub fn get_recently_played_tracks(self) -> RecentlyPlayedBuilder {
        RecentlyPlayedBuilder::new(self.0)
    }

    // Get the current queue
    ///
    /// # Scope
    /// user-read-playback-state, user-read-currently-playing
    #[cfg(all(feature = "user-read-playback-state", feature = "user-read-currently-playing"))]
    pub fn get_queue(self) -> QueueBuilder {
        QueueBuilder::new(self.0)
    }

    // Add item to queue
    ///
    /// # Scope
    /// user-modify-playback-state
    #[cfg(feature = "user-modify-playback-state")]
    pub fn add_to_queue<S: Into<String>>(self, uri_type: UriType, id: S) -> AddToQueueBuilder {
        AddToQueueBuilder::new(self.0, Uri::new(uri_type, id.into()))
    }
}
