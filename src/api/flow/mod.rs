pub mod auth;
mod credential;
pub mod creds;
pub mod pkce;

pub use auth::Flow as AuthCode;
pub use credential::Credentials;
pub use creds::Flow as Creds;
pub use pkce::Flow as Pkce;

use std::{
    collections::HashSet, fmt::Debug, future::Future, path::{Path, PathBuf}
};

use super::{OAuth, Token};
use crate::{Error, Shared};

pub trait AuthFlow: Sized + Clone {
    type Credentials;
    fn authorization_url(&self, show_dialog: bool) -> Result<String, serde_urlencoded::ser::Error>;
    fn request_access_token(&self, _auth_code: &str) -> impl Future<Output = Result<(), Error>>;
    fn scopes(&self) -> &HashSet<String>;
    fn token(&self) -> Token;
    fn set_token(&self, token: Token);
    fn refresh(&self) -> impl Future<Output = Result<(), Error>>;
    fn setup(credentials: Self::Credentials, oauth: OAuth, config: Config) -> Result<Self, Error>;
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

#[cfg(feature = "caching")]
pub trait CacheToken {
    fn id() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL while making spotify requests
    pub api_base_url: String,
    /// Path where the token is cached
    #[cfg(feature = "caching")]
    pub cache_path: PathBuf,
    /// Whether to cache the token
    #[cfg(feature = "caching")]
    pub token_cached: bool,
    /// Function to call when a new access token is generated
    pub token_callback_fn: Option<Shared<TokenCallback>>,
}

#[cfg(not(feature = "caching"))]
impl Default for Config {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.spotify.com/v1".to_string(),
            token_callback_fn: None,
        }
    }
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
    #[cfg(feature = "caching")]
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

    #[cfg(feature = "caching")]
    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    #[cfg(feature = "caching")]
    pub fn caching(&self) -> bool {
        self.token_cached
    }

    pub fn callback(&self) -> Option<Shared<TokenCallback>> {
        self.token_callback_fn.clone()
    }

    #[cfg(feature = "caching")]
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

#[cfg(feature = "caching")]
impl<S: AsRef<Path>> From<S> for Config {
    fn from(cache_dir: S) -> Self {
        Self::new(cache_dir)
    }
}
