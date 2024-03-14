use std::future::Future;
use std::sync::{Arc, Mutex};

use crate::auth::OAuth;

use super::Error;

pub mod player;
pub mod tracks;
pub mod users;

#[derive(Clone)]
pub struct Spotify {
    pub oauth: Arc<Mutex<OAuth>>,
}

pub trait SpotifyRequest {
    type Response;
    fn send(self) -> impl Future<Output=Result<Self::Response, Error>>;
}

impl Spotify {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(Self { oauth: Arc::new(Mutex::new(OAuth::new()?)) })
    }

    #[cfg(any(
    feature = "user-read-private",
    feature = "user-read-recently-played",
    feature = "user-read-playback-state",
    feature = "user-modify-playback-state",
    ))]
    pub fn player(&mut self) -> player::PlayerBuilder {
        player::PlayerBuilder::new(self.oauth.clone())
    }

    pub fn tracks(&mut self) -> tracks::TrackBuilder {
        tracks::TrackBuilder::new(self.oauth.clone())
    }

    pub fn users(&mut self) -> users::UsersBuilder {
        users::UsersBuilder::new(self.oauth.clone())
    }
}

#[macro_export]
macro_rules! query {
    ($($name: literal : $value: expr),* $(,)?) => {
        {
            use $crate::api::IntoQueryParam;
            std::collections::HashMap::from([
                $(($name, $value.into_query_param()),)*
            ])
        }
    };
}

pub trait IntoQueryParam {
    fn into_query_param(self) -> Option<String>;
}

impl<S: ToString> IntoQueryParam for Option<S> {
    fn into_query_param(self) -> Option<String> {
        self.map(|s| s.to_string())
    }
}
