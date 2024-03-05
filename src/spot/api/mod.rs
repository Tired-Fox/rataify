use crate::spot::auth::OAuth;

pub mod device;
pub mod player;

use super::Error;

pub struct Spotify {
    pub oauth: OAuth,
}

pub trait SpotifyRequest<T> {
    async fn send(self) -> Result<T, Error>;
}

impl Spotify {
    pub fn new(oauth: OAuth) -> Self {
        Self { oauth }
    }

    pub fn transfer_playback(&mut self) -> player::TransferPlaybackBuilder {
        player::TransferPlaybackBuilder::new(&mut self.oauth)
    }

    pub fn devices(&mut self) -> device::DevicesBuilder {
        device::DevicesBuilder::new(&mut self.oauth)
    }

    pub fn player_state(&mut self) -> player::PlayerStateBuilder {
        player::PlayerStateBuilder::new(&mut self.oauth)
    }
}
