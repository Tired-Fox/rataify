use crate::auth::OAuth;
use crate::{Error, SpotifyRequest, SpotifyResponse};
use crate::model::user::UserProfile;

pub struct CurrentUserProfileBuilder<'a> {
    oauth: &'a mut OAuth,
}

impl <'a> CurrentUserProfileBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self { oauth }
    }
}

impl<'a> SpotifyRequest<UserProfile> for CurrentUserProfileBuilder<'a> {
    async fn send(self) -> Result<UserProfile, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}