use std::collections::HashSet;
use std::future::Future;
use std::net::SocketAddr;
use std::str::FromStr;

use base64::Engine;
use chrono::{DateTime, Duration, Local};
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Report;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

pub use callback::Callback;

use crate::{browser, CONFIG_PATH};

use super::Credentials;

mod callback;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    token_type: String,
    scopes: HashSet<String>,
    expires: DateTime<Local>,
    access_token: String,
    refresh_token: Option<String>,
}

impl Default for AuthToken {
    fn default() -> Self {
        Self {
            token_type: String::from("Bearer"),
            scopes: HashSet::new(),
            expires: Local::now() - Duration::seconds(12),
            access_token: String::new(),
            refresh_token: None,
        }
    }
}

impl AuthToken {
    /// Check if the auth token is expired with 10 seconds leeway for slower requests
    pub fn is_expired(&self) -> bool {
        self.expires < (Local::now() - Duration::seconds(10))
    }

    /// Get the auth header for the token
    ///
    /// # Example
    ///
    /// `Bearer 1POdFZRZbvb...qqillRxMr2z`
    pub fn to_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }

    pub fn save(&self) -> color_eyre::Result<()> {
        let path = CONFIG_PATH.join("cache/token.json");
        std::fs::create_dir_all(path.parent().unwrap())?;
        Ok(std::fs::write(path, serde_json::to_string(self)?)?)
    }

    pub fn load() -> Option<Self> {
        let token = std::fs::read_to_string(CONFIG_PATH.join("cache/token.json")).ok()?;
        serde_json::from_str(&token).ok()
    }

    /// Parse a new authentication token from a string
    pub fn parse_new(value: &str) -> color_eyre::Result<Self> {
        let token: AuthToken = AuthToken::from_str(value)?;
        match token.refresh_token {
            Some(_) => {}
            None => {
                return Err(eyre!("failed to parse AuthToken::refresh_token: missing"));
            }
        }

        Ok(token)
    }

    /// Parse and refresh the token in place
    pub fn parse_refresh(&mut self, value: &str) -> color_eyre::Result<()> {
        let token: AuthToken = AuthToken::from_str(value)?;
        self.access_token = token.access_token;
        self.scopes = token.scopes;
        self.token_type = token.token_type;
        self.expires = token.expires;
        Ok(())
    }
}

impl FromStr for AuthToken {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let object: Value = serde_json::from_str(s)?;

        Ok(Self {
            token_type: object.get("token_type").ok_or_eyre("failed to parse AuthToken::token_type: missing")?
                .as_str().ok_or_eyre("failed to parse AuthToken::token_type: not a string")?
                .to_string(),
            scopes: object.get("scope").ok_or_eyre("failed to parse AuthToken::scope: missing")?
                .as_str().ok_or_eyre("failed to parse AuthToken::scope: not a space seperated string list")?
                .split(" ").map(|v| v.to_string()).collect(),
            expires: {
                let seconds = object.get("expires_in").ok_or_eyre("failed to parse AuthToken::expires: missing")?;
                let seconds = seconds.as_i64().ok_or_eyre("failed to parse AuthToken::expires: not an integer")?;
                Local::now() + Duration::seconds(seconds)
            },
            access_token: object.get("access_token").ok_or_eyre("failed to parse AuthToken::access_token: missing")?
                .as_str().ok_or_eyre("failed to parse AuthToken::access_token: not a string")?
                .to_string(),
            refresh_token: match object.get("refresh_token") {
                Some(token) => {
                    match token.as_str() {
                        Some(token) => Some(token.to_string()),
                        None => None
                    }
                }
                None => None
            },
        })
    }
}

#[derive(Debug)]
pub struct OAuth {
    pub credentials: Credentials,
    state: Uuid,
    pub scopes: HashSet<String>,
    pub token: Option<AuthToken>,
    tx: UnboundedSender<String>,
    rx: UnboundedReceiver<String>,
}

impl OAuth {
    pub fn new(credentials: Credentials, scopes: HashSet<String>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            credentials,
            state: Uuid::new_v4(),
            token: AuthToken::load(),
            scopes,
            tx,
            rx,
        }
    }

    pub fn token(&self) -> Option<&AuthToken> {
        self.token.as_ref()
    }

    /// Get a new authentication code by starting a http server to handle the spotify callback
    /// with the code, and opening the browser to the spotify login/authentication path.
    async fn new_authentication_code(&mut self) -> color_eyre::Result<String> {
        // Mini http server to serve callback and parse auth code from spotify
        let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
        let listener = TcpListener::bind(addr).await?;

        let callback = Callback::new(self.state.clone(), self.tx.clone());
        let handle = tokio::task::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);

                let handler = callback.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, handler)
                        .await
                    {
                        eprintln!("Error serving connection to spotify callback: {:?}", err);
                    }
                });
            }
        });

        // Open the default browser to the spotify login/authentication page.
        // When it is successful, the callback will be triggered and the result is returned
        browser!(
            "https://accounts.spotify.com/authorize" ?
            client_id=self.credentials.client_id,
            response_type="code",
            redirect_uri=urlencoding::encode("http://localhost:8888/Rataify/auth"),
            scope=format!("{}", self.scopes.iter().map(|v| v.clone()).collect::<Vec<_>>().join("%20")),
            state=self.state,
            show_dialog=true,
        )?;

        let result = self.rx.recv().await.ok_or(Report::msg("Spotify did not send a response"))?;
        handle.abort();
        Ok(result)
    }

    /// Get a new access token starting from getting a new authentication code
    async fn new_token(&mut self) -> color_eyre::Result<AuthToken> {
        let authentication_code = self.new_authentication_code().await?;

        let body = serde_urlencoded::to_string(&[
            ("grant_type", "authorization_code".to_string()),
            ("code", authentication_code.clone()),
            ("redirect_uri", "http://localhost:8888/Rataify/auth".to_string()),
        ])?;

        let client = reqwest::Client::new();
        let result = client.post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", self.credentials.auth()))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        let body = String::from_utf8(result.bytes().await?.to_vec())?;
        AuthToken::from_str(&body)
    }

    /// Authenticate the spotify user.
    ///
    /// Fetch a new access token and cache it
    pub async fn authenticate(&mut self) -> color_eyre::Result<()> {
        let token = self.new_token().await?;
        token.save()?;
        self.token = Some(token);
        Ok(())
    }

    pub async fn refresh(&mut self) -> color_eyre::Result<()> {
        if let Some(token) = &mut self.token {
            if let Some(refresh_token) = &token.refresh_token {
                let client = reqwest::Client::new();
                let response = client.post("https://accounts.spotify.com/api/token")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Authorization", format!("Basic {}", self.credentials.auth()))
                    .body(serde_urlencoded::to_string(&[
                        ("grant_type", "refresh_token".to_string()),
                        ("refresh_token", refresh_token.clone()),
                        ("client_id", self.credentials.client_id.clone()),
                    ])?)
                    .send()
                    .await?;

                let body = String::from_utf8(response.bytes().await?.to_vec())?;
                token.parse_refresh(&body)?;
                token.save()?;
            } else {
                eprintln!("Missing refresh token, re-authenticating...");
                self.authenticate().await?;
            }
        } else {
            eprintln!("Missing access token, re-authenticating...");
            self.authenticate().await?;
        }
        Ok(())
    }

    /// Refresh the access token if it has expired, or authenticate the user if the token is
    /// invalid or missing.
    pub async fn update(&mut self) -> color_eyre::Result<()> {
        // Check for expired token with 10-second grace period
        if let Some(token) = &mut self.token {
            // if scopes changed re-authenticate
            if !self.scopes.is_subset(&token.scopes) {
                self.authenticate().await?;
                return Ok(());
            }

            if token.expires < (Local::now() - Duration::seconds(10)) {
                self.refresh().await?;
            }
        } else {
            self.authenticate().await?;
        }
        Ok(())
    }
}