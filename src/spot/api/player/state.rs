use super::Playback;
use crate::spot::auth::OAuth;
use crate::spot::Error;
use crate::spot::{SpotifyRequest, SpotifyResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AdditionalTypes {
    #[default]
    Track,
    Episode,
}

pub struct PlayerStateBuilder<'a> {
    oauth: &'a mut OAuth,
    device_id: Option<String>,
    market: Option<String>,
    additional_types: Vec<AdditionalTypes>,
}

impl<'a> PlayerStateBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            device_id: None,
            market: None,
            additional_types: vec![AdditionalTypes::Track],
        }
    }

    pub fn device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn market<S: Into<String>>(mut self, market: S) -> Self {
        self.market = Some(market.into());
        self
    }

    pub fn additional_types<const N: usize>(
        mut self,
        additional_types: [AdditionalTypes; N],
    ) -> Self {
        self.additional_types = additional_types.to_vec();
        self
    }
}

impl<'a> SpotifyRequest<Option<Playback>> for PlayerStateBuilder<'a> {
    async fn send(mut self) -> Result<Option<Playback>, Error> {
        self.oauth.update().await?;
        let result: Result<Playback, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/player")
            .header(
                "Authorization",
                self.oauth.token.as_ref().unwrap().to_header(),
            )
            .send()
            .await
            .to_spotify_response()
            .await;

        // Map no content error to be None. This is because the playback just isn't available at the moment
        match result {
            Ok(result) => Ok(Some(result)),
            Err(Error::NoContent) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
