mod cred;

pub use cred::Credentials;

use chrono::Local;
use std::{collections::HashSet, fmt::Debug, net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;

use super::{AuthFlow, CacheToken, Callback, Config, OAuth, Token};
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
        "pkce"
    }
}

impl Flow {
    pub async fn new_authentication_code(&self) -> Result<String, Error> {
        let uri = hyper::Uri::from_str(self.oauth.redirect.as_str())?;

        // Mini http server to serve callback and parse auth code from spotify
        let addr = SocketAddr::from(([127, 0, 0, 1], uri.port_u16().unwrap_or(8888)));
        let listener = TcpListener::bind(addr).await?;

        println!("Listening on {}", addr);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let callback = Callback::new(self.oauth.state, tx);
        let handle = tokio::task::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = hyper_util::rt::TokioIo::new(stream);

                let cb = callback.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, cb)
                        .await
                    {
                        eprintln!("Error serving connection to spotify callback: {:?}", err);
                    }
                });
            }
        });

        // Open the default browser to the spotify login/authentication page.
        // When it is successful, the callback will be triggered and the result is returned
        open::that(self.authorization_url()?)?;

        let result = rx.recv().await.ok_or("Spotify did not send a response")?;
        handle.abort();
        Ok(result)
    }

    async fn new_token(&self) -> Result<(), Error> {
        let authentication_code = self.new_authentication_code().await?;

        let body = serde_urlencoded::to_string([
            ("grant_type", "authorization_code".to_string()),
            ("code", authentication_code.clone()),
            ("redirect_uri", self.oauth.redirect.clone()),
            ("client_id", self.credentials.id.clone()),
            ("code_verifier", self.credentials.verifier.to_string()),
        ])?;

        let result = reqwest::Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
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
        credentials: Credentials,
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
            flow.new_token().await?;
        } else {
            match Token::load(flow.config.cache_path(), Flow::id()) {
                Ok(token) => {
                    *flow.token.lock().unwrap() = token;
                }
                Err(err) => {
                    log::debug!("Failed to load cached token: {:?}", err);
                    flow.new_token().await?;
                }
            }
        };

        Ok(flow)
    }

    fn authorization_url(&self) -> Result<String, serde_urlencoded::ser::Error> {
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
                        .join("%20")
                ),
                ("code_challenge_method", "S256".to_string()),
                ("code_challenge", self.credentials.challenge.clone()),
            ])?
        ))
    }

    fn scopes(&self) -> &HashSet<String> {
        &self.oauth.scopes
    }

    async fn refresh(&self) -> Result<(), Error> {
        log::warn!("Refreshing token");
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
                .await?;

            let body = String::from_utf8(response.bytes().await?.to_vec())?;
            let result = self.token.lock().unwrap().parse_refresh(&body);
            if let Err(err) = result {
                log::error!("{:?}", err);
                self.new_token().await?;
                return Ok(());
            }

            let token = self.token.lock().unwrap();
            if let Some(callback) = self.config.callback() {
                callback.call(token.clone())?;
            }
            token.save(self.config.cache_path(), Flow::id())?;
        } else {
            self.new_token().await?;
        }

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
impl UserApi for Flow {}
