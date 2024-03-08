use std::future::Future;

use crate::auth::OAuth;

use super::Error;

pub mod player;
pub mod tracks;
pub mod users;

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

    pub fn users(&mut self) -> users::UsersBuilder {
        users::UsersBuilder::new(&mut self.oauth)
    }
}

/// Two-way async iterator. Mainly for use with paginated endpoints. Allows for the next and previous
/// urls to be followed automatically.
pub trait AsyncIter {
    type Item;

    fn next(&mut self) -> impl Future<Output=Option<Self::Item>>;
    fn prev(&mut self) -> impl Future<Output=Option<Self::Item>>;
}
