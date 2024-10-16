use base64::Engine;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fmt::Debug};

use super::{AuthFlow, Config, Credentials, OAuth, Token};

#[cfg(feature = "caching")]
use super::CacheToken;

use crate::{
    api::{alphabet, uuid, PublicApi, SpotifyResponse, UserApi},
    Error, Locked, Shared,
};

#[derive(Debug, Clone)]
pub struct CodeChallenge {
    pub(crate) challenge: String,
    pub(crate) verifier: String,
}

impl CodeChallenge {
    fn sha256<S: AsRef<[u8]>>(value: S) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(value);
        hasher.finalize().to_vec()
    }

    fn base64encode<S: AsRef<[u8]>>(value: S) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value)
    }

    pub fn new() -> Self {
        let verifier = uuid::<43>(alphabet::PKCE);
        let challenge = Self::base64encode(Self::sha256(&verifier));
        Self {
            verifier,
            challenge,
        }
    }

    pub fn refresh(&mut self) {
        self.verifier = uuid::<43>(alphabet::PKCE);
        self.challenge = Self::base64encode(Self::sha256(&self.verifier));
    }
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub credentials: Credentials,
    pub oauth: OAuth,
    pub(crate) config: Config,
    pub(crate) token: Shared<Locked<Token>>,
    pub(crate) code: CodeChallenge,
}

#[cfg(feature = "caching")]
impl CacheToken for Flow {
    fn id() -> &'static str {
        "pkce"
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
                code: CodeChallenge::new(),
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
                code: CodeChallenge::new(),
            })
        }
    }

    async fn request_access_token(&self, auth_code: &str) -> Result<(), Error> {
        let body = serde_urlencoded::to_string([
            ("grant_type", "authorization_code".to_string()),
            ("code", auth_code.to_string()),
            ("redirect_uri", self.oauth.redirect.clone()),
            ("client_id", self.credentials.id.clone()),
            ("code_verifier", self.code.verifier.to_string()),
        ])?;

        let result = reqwest::Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
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
        Ok(format!(
            "https://accounts.spotify.com/authorize?{}",
            serde_urlencoded::to_string([
                ("response_type", "code".to_string()),
                ("client_id", self.credentials.id.clone()),
                ("redirect_uri", self.oauth.redirect.clone()),
                ("state", self.oauth.state.to_string()),
                (
                    "scope",
                    self.oauth
                        .scopes
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                ("code_challenge_method", "S256".to_string()),
                ("code_challenge", self.code.challenge.clone()),
            ])?
        ))
    }

    fn scopes(&self) -> &HashSet<String> {
        &self.oauth.scopes
    }

    async fn refresh(&self) -> Result<(), Error> {
        // If scopes aren't matching then the token should not be refreshed
        let refresh_token = self.token.lock().unwrap().refresh_token.clone();
        if let Some(refresh_token) = refresh_token {
            let client = reqwest::Client::new();
            let response = client
                .post("https://accounts.spotify.com/api/token")
                .header("Content-Type", "application/x-www-form-urlencoded")
                //.header("Authorization", format!("Basic {}", self.credentials))
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
            self.token.lock().unwrap().parse_refresh(&body).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;
            {
                let token = self.token.lock().unwrap();

                #[cfg(feature = "caching")]
                if self.config.caching() {
                    token.save(self.config.cache_path(), Flow::id()).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;
                }

                if let Some(callback) = self.config.callback() {
                    callback.call(token.clone()).map_err(|e| Error::refresh(e, self.oauth.redirect.clone(), self.oauth.state.clone()))?;
                }
            }
        } else {
            return Err(Error::refresh("Missing refresh token", self.oauth.redirect.clone(), self.oauth.state.clone()));
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
impl PublicApi for Flow {}
impl UserApi for Flow {}
