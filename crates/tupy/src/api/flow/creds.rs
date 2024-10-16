use std::{collections::HashSet, fmt::Debug};

use super::{AuthFlow, Config, OAuth, Token, Credentials};

#[cfg(feature = "caching")]
use super::CacheToken;

use crate::{
    api::{PublicApi, SpotifyResponse},
    Error, Locked, Shared,
};

#[derive(Debug, Clone)]
pub struct Flow {
    pub(crate) credentials: Credentials,
    pub(crate) oauth: OAuth,
    pub(crate) config: Config,
    pub(crate) token: Shared<Locked<Token>>,
}

#[cfg(feature = "caching")]
impl CacheToken for Flow {
    fn id() -> &'static str {
        "creds"
    }
}

impl AuthFlow for Flow {
    type Credentials = Credentials;

    fn setup(
        credentials: Self::Credentials,
        oauth: OAuth,
        config: Config,
    ) -> Result<Self, Error> {
        #[cfg(feature = "caching")]
        {
            let flow = Self {
                config,
                token: Shared::new(Locked::new(Token::default())),
                credentials,
                oauth,
            };

            if flow
                .config
                .cache_path()
                .join(format!("spotify.{}.token", Flow::id()))
                .exists()
                || flow.config.caching()
            {
                match Token::load(flow.config.cache_path(), Flow::id()) {
                    Ok(token) => {
                        *flow.token.lock().unwrap() = token;
                    },
                    _ => log::debug!("Failed to load cached token"),
                }
            };
            Ok(flow)
        }
        #[cfg(not(feature = "caching"))]
        {
            Ok(Self {
                config,
                token: Shared::new(Locked::new(Token::default())),
                credentials,
                oauth,
            })
        }
    }

    async fn request_access_token(&self, _auth_code: &str) -> Result<(), Error> {
        let body = serde_urlencoded::to_string([("grant_type", "client_credentials".to_string())])?;

        let result = reqwest::Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Basic {}", self.credentials))
            .body(body)
            .send()
            .await?;

        let token = Token::from_auth(SpotifyResponse::from_response(result).await?)?;

        #[cfg(feature = "caching")]
        if self.config.caching() {
            token.save(self.config.cache_path(), Flow::id())?;
        }

        if let Some(callback) = self.config.callback() {
            callback.call(token.clone())?;
        }

        *self.token.lock().unwrap() = token;
        Ok(())
    }

    fn authorization_url(&self, _show_dialog: bool) -> Result<String, serde_urlencoded::ser::Error> {
        Ok(String::new())
    }

    fn scopes(&self) -> &HashSet<String> {
        &self.oauth.scopes
    }

    async fn refresh(&self) -> Result<(), Error> {
        Err(Error::refresh("Missing refresh token", self.oauth.redirect.clone(), self.oauth.state.clone()))
    }

    fn token(&self) -> Token {
        self.token.lock().unwrap().clone()
    }

    fn set_token(&self, token: Token) {
        *self.token.lock().unwrap() = token;
    }
}

// API Implementations for this specific workflow
impl PublicApi for Flow {}
