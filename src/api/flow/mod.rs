pub mod auth;
pub mod creds;
pub mod pkce;
mod credential;

pub use credential::Credentials;
pub use auth::Flow as AuthCode;
pub use pkce::Flow as Pkce;
pub use creds::Flow as Creds;

use std::{collections::HashSet, fmt::Debug, future::Future, path::{Path, PathBuf}, pin::Pin};

use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, service::Service, Method, Request, Response};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::{OAuth, Token};
use crate::{Error, Shared};

pub trait AuthFlow: Sized + Clone {
    type Credentials;
    fn authorization_url(&self) -> Result<String, serde_urlencoded::ser::Error>;
    fn scopes(&self) -> &HashSet<String>;
    fn token(&self) -> impl Future<Output=Result<Token, Error>>;
    fn refresh(&self) -> impl Future<Output=Result<(), Error>>;
    fn setup(credentials: Self::Credentials, oauth: OAuth, config: Config) -> impl Future<Output = Result<Self, Error>>;
}

pub struct TokenCallback(pub Shared<dyn Fn(Token) -> Result<(), Error> + Send + Sync>);

impl Debug for TokenCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn<TokenCallback>()")
    }
}

impl TokenCallback {
    pub fn new<F: Fn(Token) -> Result<(), Error> + 'static + Send + Sync>(f: F) -> Self {
        Self(Shared::new(f))
    }

    pub fn call(&self, token: Token) -> Result<(), Error> {
        (self.0)(token)
    }
}

fn layout<S: AsRef<str>>(body: S) -> String {
    format!(
        indoc::indoc! {r#"
            <html>
                <head>
                    <title>Rataify</title>
                    <style>
                    * {{
                        box-sizing: border-box
                    }}
                    html {{
                        font-family: Arial;
                        background-color: #191414;
                        color: #FFFFFF
                    }}
                    :is(h1, h3) {{
                        text-align: center;
                    }}
                    body {{
                        padding: 1.5rem;
                    }}
                    .green {{
                        color: #1DB954
                    }}
                    </style>
                </head>
                <body>
                    {}
                </body>
            </html>
        "#},
        body.as_ref()
    )
}

pub trait CacheToken {
    fn id() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL while making spotify requests
    pub api_base_url: String,
    /// Path where the token is cached
    pub cache_path: PathBuf,
    /// Whether to cache the token
    pub token_cached: bool,
    /// Function to call when a new access token is generated
    pub token_callback_fn: Option<Shared<TokenCallback>>,
}

impl Config {
    /// Creates a new configuration for a Spotify Authentication Code flow
    ///
    /// The cache dir is usually the name of the application. However, this path is just any path
    /// relative to the systems native cache directory.
    ///
    /// Windows: `%LocalAppData%\<cache_dir>`
    /// Linux: `$XDG_CACHE_HOME/<cache_dir>` or `$HOME/.cache/<cache_dir>`
    /// MacOs: `$HOME/Library/Caches/<cache_dir>`
    pub fn new<S>(cache_dir: S) -> Self
    where
        S: AsRef<Path>,
    {
        Self {
            api_base_url: "https://api.spotify.com/v1".to_string(),
            cache_path: dirs::cache_dir().unwrap().join(cache_dir),
            token_cached: true,
            token_callback_fn: None,
        }
    }

    pub fn api_url(&self) -> &str {
        &self.api_base_url
    }

    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    pub fn caching(&self) -> bool {
        self.token_cached
    }

    pub fn callback(&self) -> Option<Shared<TokenCallback>> {
        self.token_callback_fn.clone()
    }

    pub fn with_caching(mut self, state: bool) -> Self {
        self.token_cached = state;
        self
    }

    pub fn with_callback<F>(mut self, f: F) -> Self
    where
        F: Fn(Token) -> Result<(), Error> + 'static + Send + Sync,
    {
        self.token_callback_fn = Some(Shared::new(TokenCallback::new(f)));
        self
    }
}

impl<S: AsRef<Path>> From<S> for Config {
    fn from(cache_dir: S) -> Self {
        Self::new(cache_dir)
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct AuthResponse {
    pub(crate) code: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) state: String,
}

pub struct Callback {
    pub(crate) state: String,
    pub(crate) tx: UnboundedSender<String>,
}

impl Clone for Callback {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl Callback {
    pub fn new(uuid: Uuid, tx: UnboundedSender<String>) -> Self {
        Self {
            state: uuid.to_string(),
            tx,
        }
    }

    fn handler(
        query: Option<&str>,
        state: String,
        result: UnboundedSender<String>,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
        match query {
            Some(query) => {
                let response: AuthResponse = serde_qs::from_str(query)?;
                if let Some(err) = response.error {
                    return Err(err.into());
                }

                // Validate State for cross-site request forgery
                match response.state == state {
                    false => {
                        result.send(String::new()).unwrap();
                        Err("Invalid response state".into())
                    }
                    true => {
                        result.send(response.code.unwrap()).unwrap();
                        Ok(Response::builder()
                            .body(Full::new(Bytes::from(layout(indoc::indoc! {r#"
                                    <h1>
                                       Successfully granted access to
                                       <span class="green">Spotify</span>
                                        for Rotify
                                    </h1>
                                    <h3>This tab may now be closed</h3>
                                "#}))))
                            .unwrap())
                    }
                }
            }
            None => {
                result.send(String::new()).unwrap();
                Err("Spotify did not send a response".into())
            }
        }
    }
}

impl Service<Request<Incoming>> for Callback {
    type Response = Response<Full<Bytes>>;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        match (req.method().clone(), req.uri().path()) {
            (Method::GET, "/Rataify/auth") => {
                let state = self.state.clone();
                let tx = self.tx.clone();
                Box::pin(async move {
                    match Callback::handler(req.uri().query(), state, tx) {
                        Ok(response) => Ok(response),
                        Err(err) => {
                            log::error!("{:?}", err);
                            Ok(Response::builder()
                                .status(500)
                                .body(Full::new(Bytes::from("<h1>500 Internal Server Error<h1>")))
                                .unwrap())
                        }
                    }
                })
            }
            _ => Box::pin(async {
                Ok(Response::builder()
                    .status(404)
                    .body(Full::new(Bytes::from(layout(indoc::indoc! {r#"
                                <h1>"404 Page not found"</h1>
                            "#}))))
                    .unwrap())
            }),
        }
    }
}
