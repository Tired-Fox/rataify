use std::future::Future;
use crate::auth::OAuth;
use crate::prompt::prompt_creds_if_missing;

pub mod player;
pub mod tracks;
pub mod users;

use super::Error;

pub struct Spotify {
    pub oauth: OAuth,
}

pub trait SpotifyRequest<T> {
    fn send(self) -> impl Future<Output=Result<T, Error>>;
}

impl Spotify {
    pub fn new() -> Self {
        Self { oauth: OAuth::new() }
    }

    #[cfg(any(
    feature = "user-read-private",
    feature = "user-read-recently-played",
    feature = "user-read-playback-state",
    feature = "user-modify-playback-state",
    ))]
    pub fn player(&mut self) -> player::PlayerBuilder {
        player::PlayerBuilder::new(&mut self.oauth)
    }

    pub fn tracks(&mut self) -> tracks::TrackBuilder {
        tracks::TrackBuilder::new(&mut self.oauth)
    }
}
