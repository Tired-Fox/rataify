mod cred;

pub use cred::Credentials;

use chrono::Local;
use std::{collections::HashSet, fmt::Debug, net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;

use super::{AuthFlow, CacheToken, Callback, Config, OAuth, Token};
use crate::{
    api::{SpotifyResponse, UserApi},
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

impl Flow {
    /// Get a list of steps to setup the authentication code flow
    ///
    /// The steps include what url to go to, how to rcreate an app, and how to find the client id
    /// and secret.
    /// TODO: Remove before use in final product
    #[cfg(feature = "cli")]
    pub fn steps() -> Vec<&'static str> {
        vec![
            "Go to \x1b[36mhttps://developer.spotify.com/dashboard\x1b[39m",
            "Click the 'Create app' button",
            "Name the app whatever you want",
            "Add a \x1b[33mRedirect URI\x1b[39m of \x1b[36mhttp://localhost:\x1b[0m{\x1b[33mport\x1b[0m}{\x1b[33many sub path\x1b[0m}",
            "Enable the following API/SDKs: 'Web API' and 'Web Playback SDK'",
            "Agree to Spotify's \x1b]8;;https://developer.spotify.com/terms\x1b\\\x1b[36mDeveloper Terms of Service\x1b[39m\x1b]8;;\x1b\\ and \x1b]8;;https://developer.spotify.com/documentation/design\x1b\\\x1b[36mDesign Guidelines\x1b[39m\x1b]8;;\x1b\\",
            "After you are redirected, click on the settings button on the top right",
            "Click 'View client secret'",
        ]
    }

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
        ])?;

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
                        .join("%20")
                ),
                ("redirect_uri", self.oauth.redirect.clone()),
                ("state", self.oauth.state.to_string()),
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
                .header("Authorization", format!("Basic {}", self.credentials))
                .body(serde_urlencoded::to_string(&[
                    ("grant_type", "refresh_token".to_string()),
                    ("refresh_token", refresh_token.clone()),
                    ("client_id", self.credentials.id.clone()),
                ])?)
                .send()
                .await?;

            let body = String::from_utf8(response.bytes().await?.to_vec())?;
            {
                let mut token = self.token.lock().unwrap();
                token.parse_refresh(&body)?;
                if self.config.caching() {
                    token.save(self.config.cache_path(), Flow::id())?;
                }
                if let Some(callback) = self.config.callback() {
                    callback.call(token.clone())?;
                }
            }
        } else {
            log::error!("Missing refresh token, re-authenticating...");
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
impl UserApi for Flow {}
