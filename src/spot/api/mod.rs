use crate::spot::auth::OAuth;

pub mod player;

pub struct Spotify {
    pub oauth: OAuth,
}

pub trait SpotifyRequest {
    async fn send(self) -> color_eyre::Result<reqwest::Response>;
}

impl Spotify {
    pub fn new(oauth: OAuth) -> Self {
        Self {
            oauth
        }
    }

    pub fn player_state(&mut self) -> player::PlayerStateBuilder {
        player::PlayerStateBuilder::new(&mut self.oauth)
    }
}
