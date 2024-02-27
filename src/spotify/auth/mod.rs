use std::collections::HashSet;
use std::convert::Infallible;
use std::env::home_dir;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;

use base64::Engine;
use chrono::{DateTime, Duration, Local, Timelike};
use color_eyre::eyre::OptionExt;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Report;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, Response};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::browser;

use super::{Credentials, SendRequest};

mod cache;

#[derive(Debug, Deserialize)]
struct AuthCodeResponse {
    code: Option<String>,
    error: Option<String>,
    state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    token_type: String,
    scope: HashSet<String>,
    expires: DateTime<Local>,
    access_token: String,
    refresh_token: String,
}

impl Default for AuthToken {
    fn default() -> Self {
        Self {
            token_type: String::from("Bearer"),
            scope: HashSet::new(),
            expires: Local::now() - Duration::seconds(12),
            access_token: String::new(),
            refresh_token: String::new(),
        }
    }
}

impl AuthToken {
    /// Check if the auth token is expired with 10 seconds leeway for slower requests
    pub fn is_expired(&self) -> bool {
        self.expires < (Local::now() - Duration::seconds(10))
    }

    /// Get the auth header for the token
    pub fn to_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }

    pub fn save(&self) -> color_eyre::Result<()> {
        #[cfg(windows)]
        return {
            let path = home::home_dir().unwrap().join(".rataify/cache.json");
            std::fs::create_dir_all(path.parent().unwrap())?;
            Ok(std::fs::write(path, serde_json::to_string(self)?)?)
        };
        #[cfg(not(windows))]
        return {
            let path = home::home_dir().unwrap().join(".config/rataify/cache.json");
            std::fs::create_dir_all(path.parent().unwrap())?;
            Ok(std::fs::write(path, serde_json::to_string(self)?)?);
        };
    }

    pub fn load() -> Option<Self> {
        #[cfg(windows)]
        let token = std::fs::read_to_string(home::home_dir().unwrap().join(".rataify/cache.json")).unwrap_or(String::new());
        #[cfg(not(windows))]
        let token = std::fs::read_to_string(home::home_dir().unwrap().join(".config/rataify/cache.json")).unwrap_or(String::new());

        serde_json::from_str(&token).ok()
    }
}

impl FromStr for AuthToken {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let object: Value = serde_json::from_str(s)?;
        let msg = "Failed to parse AuthToken from response";

        Ok(Self {
            token_type: object.get("token_type").ok_or_eyre(msg)?.as_str().ok_or_eyre(msg)?.to_string(),
            scope: object.get("scope").ok_or_eyre(msg)?.as_str().ok_or_eyre(msg)?.split(" ").map(|v| v.to_string()).collect(),
            expires: {
                let seconds = object.get("expires_in").ok_or_eyre(msg)?;
                let seconds = seconds.as_i64().ok_or_eyre(msg)?;
                Local::now() + Duration::seconds(seconds)
            },
            access_token: object.get("access_token").ok_or_eyre(msg)?.as_str().ok_or_eyre(msg)?.to_string(),
            refresh_token: object.get("refresh_token").ok_or_eyre(msg)?.as_str().ok_or_eyre(msg)?.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct OAuth {
    creds: Credentials,
    state: Uuid,
    scopes: HashSet<String>,
    token: Option<AuthToken>,
    tx: UnboundedSender<String>,
    rx: UnboundedReceiver<String>,
}

impl OAuth {
    pub fn new(credentials: Credentials, scopes: HashSet<String>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            creds: credentials,
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

    async fn handler(req: Request<Incoming>, uuid: Uuid, result: UnboundedSender<String>) -> color_eyre::Result<Response<Full<Bytes>>> {
        match (req.method().clone(), req.uri().path().clone()) {
            (Method::GET, "/Rataify/auth") => {
                match req.uri().query() {
                    Some(query) => {
                        let response: AuthCodeResponse = serde_qs::from_str(query)?;
                        if let Some(err) = response.error {
                            return Err(Report::msg(err));
                        }
                        // Validate State for cross-site request forgery
                        match response.state == uuid.to_string() {
                            false => {
                                result.send(String::new()).unwrap();
                                return Err(Report::msg("Invalid response state"));
                            }
                            true => {
                                result.send(response.code.unwrap()).unwrap();
                                Ok(
                                    Response::builder()
                                        .body(Full::new(Bytes::from("<h1>Successfully Logged In to Rataify</h1>")))
                                        .unwrap()
                                )
                            }
                        }
                    }
                    None => {
                        result.send(String::new()).unwrap();
                        return Err(Report::msg("Spotify did not send a response"));
                    }
                }
            }
            _ => {
                result.send(String::new()).unwrap();
                Ok(
                    Response::builder()
                        .status(404)
                        .body(Full::new(Bytes::from("<h1>404 Page not found<h1>")))
                        .unwrap()
                )
            }
        }
    }

    async fn new_authentication_code(&mut self) -> color_eyre::Result<String> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
        let listener = TcpListener::bind(addr).await?;

        let state = self.state.clone();
        let tx2 = self.tx.clone();

        let handle = tokio::task::spawn(async move {
            let tx = tx2.clone();
            let uuid = state.clone();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);

                let tx = tx.clone();
                let uuid = uuid.clone();
                let callback = move |req: Request<Incoming>| -> Pin<Box<dyn Future<Output=Result<Response<Full<Bytes>>, Infallible>> + Send + 'static>> {
                    let tx = tx.clone();
                    let uuid = uuid.clone();
                    Box::pin(async move {
                        match OAuth::handler(req, uuid, tx).await {
                            Err(err) => {
                                Ok(Response::builder()
                                    .status(500)
                                    .body(Full::new(Bytes::from(format!("<h1>500 Internal Server Error</h1>\n<h3>{err}</h3>"))))
                                    .unwrap()
                                )
                            }
                            Ok(res) => Ok(res)
                        }
                    })
                };

                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(callback))
                        .await
                    {
                        eprintln!("Error serving connection to spotify callback: {:?}", err);
                    }
                });
            }
        });

        browser!(
            "https://accounts.spotify.com/authorize" ?
            client_id=self.creds.client_id,
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

    async fn new_token(&mut self) -> color_eyre::Result<AuthToken> {
        let authentication_code = self.new_authentication_code().await?;

        let body = serde_urlencoded::to_string(&[
            ("grant_type", "authorization_code".to_string()),
            ("code", authentication_code.clone()),
            ("redirect_uri", "http://localhost:8888/Rataify/auth".to_string()),
        ])?;

        let client = reqwest::Client::new();
        let result = client.post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", self.creds.auth()))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        let body = String::from_utf8(result.bytes().await?.to_vec())?;
        AuthToken::from_str(&body)
    }

    pub async fn authenticate(&mut self) -> color_eyre::Result<()> {
        let token = self.new_token().await?;
        token.save()?;
        self.token = Some(token);
        Ok(())
    }

    pub async fn refresh(&mut self) -> color_eyre::Result<()> {
        // Check for expired token with 10 second grace period
        if let Some(token) = &self.token {
            if token.expires < (Local::now() - Duration::seconds(10)) {
                println!("REFRESHED");
                let client = reqwest::Client::new();
                let response = client.post("https://accounts.spotify.com/api/token")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(serde_urlencoded::to_string(&[
                        ("grant_type", "refresh_token".to_string()),
                        ("refresh_token", token.refresh_token.clone()),
                        ("client_id", self.creds.client_id.clone()),
                    ])?)
                    .send()
                    .await?;

                let body = String::from_utf8(response.bytes().await?.to_vec())?;
                self.token = Some(AuthToken::from_str(&body)?);
            }
        } else {
            println!("CREATED");
            self.authenticate().await?;
        }
        Ok(())
    }
}