use std::{collections::HashSet, fmt::Debug};

use super::{AuthFlow, CacheToken, Config, Credentials, OAuth, Token};
use crate::{
    api::{PublicApi, SpotifyResponse, UserApi},
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
        "auth-code"
    }
}

impl AuthFlow for Flow {
    type Credentials = Credentials;

    fn setup(credentials: Credentials, oauth: OAuth, config: Config) -> Result<Self, Error> {
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

    async fn request_access_token(&self, auth_code: &str) -> Result<(), Error> {
        let body = serde_urlencoded::to_string([
            ("grant_type", "authorization_code".to_string()),
            ("code", auth_code.to_string()),
            ("redirect_uri", self.oauth.redirect.clone()),
        ])?;

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

    fn authorization_url(&self, show_dialog: bool) -> Result<String, serde_urlencoded::ser::Error> {
        Ok(format!(
            "https://accounts.spotify.com/authorize?{}",
            serde_urlencoded::to_string([
                ("response_type", "code".to_string()),
                ("client_id", self.credentials.id.clone()),
                (
                    "scope",
                    self.oauth
                        .scopes
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                ("redirect_uri", self.oauth.redirect.clone()),
                ("state", self.oauth.state.to_string()),
                ("show_dialog", show_dialog.to_string()),
            ])?
        ))
    }

    fn scopes(&self) -> &HashSet<String> {
        &self.oauth.scopes
    }

    async fn refresh(&self) -> Result<(), Error> {
        let refresh_token = self.token.lock().unwrap().refresh_token.clone();
        if let Some(refresh_token) = refresh_token {
            let client = reqwest::Client::new();
            let response = client
                .post("https://accounts.spotify.com/api/token")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Authorization", format!("Basic {}", self.credentials))
                .body(serde_urlencoded::to_string(&[
                    ("grant_type", "refresh_token".to_string()),
                    ("refresh_token", refresh_token.clone()),
                    ("client_id", self.credentials.id.clone()),
                ])?)
                .send()
                .await.map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;

            let body = String::from_utf8(
                response.bytes().await.map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?
                    .to_vec()).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;
            {
                let mut token = self.token.lock().unwrap();
                token.parse_refresh(&body).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;

                #[cfg(feature = "caching")]
                if self.config.caching() {
                    token.save(self.config.cache_path(), Flow::id()).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;
                }

                if let Some(callback) = self.config.callback() {
                    callback.call(token.clone()).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;
                }
            }
        } else {
            return Err(Error::refresh("Missing refresh token", self.oauth.redirect.clone(), self.oauth.state.clone()))
        }
        Ok(())
    }

    fn token(&self) -> Token {
        self.token.lock().unwrap().clone()
    }

    fn set_token(&self, token: Token) {
        *self.token.lock().unwrap() = token;
    }
}

// API Implementations for this specific workflow
impl UserApi for Flow {}
impl PublicApi for Flow {}
