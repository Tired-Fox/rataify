pub mod auth;
pub mod flow;

pub mod request;
pub mod response;
pub(crate) mod markets;
mod user;
mod public;

use std::collections::{HashMap, HashSet};

use flow::AuthFlow;
use auth::{OAuth, Token};
use hyper::{
    header::{HeaderName, HeaderValue},
    Method, StatusCode,
};

pub use user::UserApi;
pub use public::PublicApi;

use crate::{Error, SpotifyErrorType};

use self::request::IntoSpotifyId;

pub(crate) static API_BASE_URL: &str = "https://api.spotify.com/v1";

pub type DefaultResponse = HashMap<String, serde_json::Value>;

/// Wrapper to build and send spotify requests using `reqwest`
pub struct SpotifyRequest<B: Into<reqwest::Body>> {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<HeaderName, String>,
    pub params: HashMap<String, String>,
    pub body: Option<B>,
}

#[derive(Debug)]
pub struct SpotifyResponse {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl SpotifyResponse {
    async fn from_response(response: reqwest::Response) -> Result<Self, Error> {
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();
        let body = String::from_utf8(response.bytes().await?.to_vec())?;
        match status.is_success() {
            true => Ok(SpotifyResponse {
                status,
                headers,
                body,
            }),
            _ => {
                if !body.is_empty() {
                    let err_res: DefaultResponse =
                        serde_json::from_str(&body).map_err(Error::custom)?;

                    if err_res.contains_key("error_description") {
                        Err(Error::Auth {
                            code: status.as_u16(),
                            error: err_res.get("error").unwrap().as_str().unwrap().to_owned(),
                            message: err_res.get("error_description").unwrap().as_str().unwrap().to_owned(),
                        })
                    } else {
                        let err = err_res.get("error").unwrap().as_object().unwrap();
                        Err(Error::Request {
                            error_type: SpotifyErrorType::from(status.clone()),
                            code: status.as_u16(),
                            message: err.get("message").unwrap().as_str().unwrap().to_owned(),
                        })
                    }
                } else {
                    Err(Error::Request {
                        error_type: SpotifyErrorType::from(status.clone()),
                        code: status.as_u16(),
                        message: "Failed to make spotify request".to_owned(),
                    })
                }
            }
        }
    }
}

impl SpotifyRequest<String> {
    pub fn new<S: AsRef<str>>(method: Method, url: S) -> Self {
        Self {
            method,
            url: url.as_ref().to_string(),
            headers: HashMap::new(),
            params: HashMap::new(),
            body: None,
        }
    }
}

pub trait IntoSpotifyParam {
    fn into_spotify_param(self) -> Option<String>;
}

impl<I: IntoSpotifyId> IntoSpotifyParam for I {
    fn into_spotify_param(self) -> Option<String> {
        Some(self.into_spotify_id())
    }
}

impl IntoSpotifyParam for Option<()> {
    fn into_spotify_param(self) -> Option<String> {
        self.map(|_| String::new())
    }
}

impl<B: Into<reqwest::Body>> SpotifyRequest<B> {
    pub fn header<V: AsRef<str>>(mut self, key: HeaderName, value: V) -> Self {
        self.headers.insert(key, value.as_ref().to_string());
        self
    }

    pub fn headers<V: AsRef<str>, I: IntoIterator<Item = (HeaderName, V)>>(
        mut self,
        items: I,
    ) -> Self {
        self.headers
            .extend(items.into_iter().map(|(k, v)| (k, v.as_ref().to_string())));
        self
    }

    pub fn param<K: AsRef<str>, V: IntoSpotifyParam>(mut self, key: K, value: V) -> Self {
        if let Some(value) = value.into_spotify_param() {
            self.params.insert(key.as_ref().to_string(), value);
        }
        self
    }

    pub fn params<K: AsRef<str>, V: IntoSpotifyParam, I: IntoIterator<Item = (K, V)>>(
        mut self,
        items: I,
    ) -> Self {
        self.params.extend(
            items
                .into_iter()
                .filter_map(|(k, v)| {
                    let v = v.into_spotify_param()?;
                    Some((k.as_ref().to_string(), v))
                }),
        );
        self
    }

    pub fn body<S: Into<reqwest::Body>>(self, body: S) -> SpotifyRequest<S> {
        SpotifyRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            params: self.params,
            body: Some(body),
        }
    }

    pub async fn send_raw(mut self, token: Token) -> Result<SpotifyResponse, Error> {
        let url = if !self.params.is_empty() {
            format!(
                "{}?{}",
                self.url,
                serde_urlencoded::to_string(self.params)?,
            )
        } else {
            self.url
        };

        let mut request = match self.method {
            Method::GET => reqwest::Client::new().get(url),
            Method::POST => reqwest::Client::new().post(url),
            Method::PUT => reqwest::Client::new().put(url),
            Method::DELETE => reqwest::Client::new().delete(url),
            _ => unimplemented!(),
        }
        .headers(
            self.headers
                .drain()
                .map(|(k, v)| (k, v.parse::<HeaderValue>().unwrap()))
                .collect(),
        )
        .header(
            "Authorization",
            format!("{} {}", token.ttype(), token.access()),
        );

        if let Some(body) = self.body {
            request = request.body(body);
        } else {
            request = request.body("").header("Content-Length", 0);
        }

        SpotifyResponse::from_response(request.send().await?).await
    }

    pub async fn send(mut self, token: Token) -> Result<SpotifyResponse, Error> {
        self.url = format!("{}/{}", API_BASE_URL, self.url);
        self.send_raw(token).await
    }
}

#[derive(Debug, Clone)]
pub struct Spotify<T: AuthFlow> {
    pub api: T,
}

impl<F: AuthFlow> Spotify<F> {
    pub async fn new<I: Into<flow::Config>>(
        credentials: F::Credentials,
        oauth: OAuth,
        config: I,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            api: F::setup(credentials, oauth, config.into()).await?,
        })
    }
}

pub fn validate_scope(scopes: &HashSet<String>, required: &[&'static str]) -> Result<(), Error> {
    let missing = required
        .iter()
        .filter(|scope| !scopes.contains(**scope))
        .map(|scope| scope.to_string())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(Error::ScopesNotGranted(missing));
    }
    Ok(())
}

pub mod scopes {
    pub static USER_READ_EMAIL: &str = "user-read-email";
    pub static USER_READ_PRIVATE: &str = "user-read-private";
    pub static USER_TOP_READ: &str = "user-top-read";
    pub static USER_READ_RECENTLY_PLAYED: &str = "user-read-recently-played";
    pub static USER_FOLLOW_READ: &str = "user-follow-read";
    pub static USER_LIBRARY_READ: &str = "user-library-read";
    pub static USER_READ_CURRENTLY_PLAYING: &str = "user-read-currently-playing";
    pub static USER_READ_PLAYBACK_STATE: &str = "user-read-playback-state";
    pub static USER_READ_PLAYBACK_POSITION: &str = "user-read-playback-position";
    pub static PLAYLIST_READ_COLLABORATIVE: &str = "playlist-read-collaborative";
    pub static PLAYLIST_READ_PRIVATE: &str = "playlist-read-private";
    pub static USER_FOLLOW_MODIFY: &str = "user-follow-modify";
    pub static USER_LIBRARY_MODIFY: &str = "user-library-modify";
    pub static USER_MODIFY_PLAYBACK_STATE: &str = "user-modify-playback-state";
    pub static PLAYLIST_MODIFY_PUBLIC: &str = "playlist-modify-public";
    pub static PLAYLIST_MODIFY_PRIVATE: &str = "playlist-modify-private";
    pub static UGC_IMAGE_UPLOAD: &str = "ugc-image-upload";
}
