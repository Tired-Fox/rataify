use chrono::Local;
use std::{collections::HashSet, fmt::Debug};

use super::{AuthFlow, CacheToken, Config, OAuth, Token, Credentials};
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

impl CacheToken for Flow {
    fn id() -> &'static str {
        "creds"
    }
}

impl Flow {
    async fn new_token(&self) -> Result<(), Error> {
        let body = serde_urlencoded::to_string([("grant_type", "client_credentials".to_string())])?;

        let result = reqwest::Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Basic {}", self.credentials))
            .body(body)
            .send()
            .await?;

        let token = Token::from_auth(SpotifyResponse::from_response(result).await?)?;
        if self.config.caching() {
            token.save(self.config.cache_path(), Flow::id())?;
        }

        if let Some(callback) = self.config.callback() {
            callback.call(token.clone())?;
        }

        *self.token.lock().unwrap() = token;
        Ok(())
    }
}

impl AuthFlow for Flow {
    type Credentials = Credentials;

    async fn setup(
        credentials: Self::Credentials,
        oauth: OAuth,
        config: Config,
    ) -> Result<Self, Error> {
        let flow = Self {
            config,
            token: Shared::new(Locked::new(Token::default())),
            credentials,
            oauth,
        };

        if !flow
            .config
            .cache_path()
            .join(format!("spotify.{}.token", Flow::id()))
            .exists()
            || !flow.config.caching()
        {
            flow.new_token().await?
        } else {
            match Token::load(flow.config.cache_path(), Flow::id()) {
                Ok(token) => *flow.token.lock().unwrap() = token,
                Err(err) => {
                    log::error!("Failed to load cached token: {:?}", err);
                    flow.new_token().await?
                }
            }
        };

        Ok(flow)
    }

    fn authorization_url(&self) -> Result<String, serde_urlencoded::ser::Error> {
        Ok(String::new())
    }

    fn scopes(&self) -> &HashSet<String> {
        &self.oauth.scopes
    }

    async fn refresh(&self) -> Result<(), Error> {
        self.new_token().await?;
        Ok(())
    }

    async fn token(&self) -> Result<Token, Error> {
        if self.token.lock().unwrap().scopes != self.oauth.scopes {
            self.new_token().await?;
        } else if self.token.lock().unwrap().expires <= Local::now() {
            self.refresh().await?;
        }

        Ok(self.token.lock().unwrap().clone())
    }
}

// API Implementations for this specific workflow
impl PublicApi for Flow {}
