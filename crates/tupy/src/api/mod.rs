pub mod auth;
pub mod flow;

mod public;
pub mod request;
pub mod response;
mod user;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display, str::FromStr,
};

pub use auth::{OAuth, Token};
use flow::AuthFlow;
use reqwest::{StatusCode, Method, header::{HeaderName, HeaderValue}};

pub use public::PublicApi;
use serde::{Deserialize, Deserializer};
pub use user::UserApi;

use crate::{Error, SpotifyErrorType};

pub(crate) static API_BASE_URL: &str = "https://api.spotify.com/v1";

pub type DefaultResponse = HashMap<String, serde_json::Value>;

/// Wrapper to build and send spotify requests using `reqwest`
pub(crate) struct SpotifyRequest<B: Into<reqwest::Body>> {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<HeaderName, String>,
    pub params: HashMap<String, String>,
    pub body: Option<B>,
}

#[derive(Debug)]
pub struct SpotifyResponse {
    status: StatusCode,
    #[allow(dead_code)]
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
                            message: err_res
                                .get("error_description")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_owned(),
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

macro_rules! impl_into_spotify_param {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoSpotifyParam for $ty {
                fn into_spotify_param(self) -> Option<String> {
                    Some(self.to_string())
                }
            }
        )*
    }
}

macro_rules! impl_into_spotify_param_with_ref {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoSpotifyParam for $ty {
                fn into_spotify_param(self) -> Option<String> {
                    Some(self.to_string())
                }
            }

            impl IntoSpotifyParam for &$ty {
                fn into_spotify_param(self) -> Option<String> {
                    Some(self.to_string())
                }
            }
        )*
    }
}

pub trait IntoSpotifyParam {
    fn into_spotify_param(self) -> Option<String>;
}

impl IntoSpotifyParam for Option<()> {
    fn into_spotify_param(self) -> Option<String> {
        self.map(|_| String::new())
    }
}

impl_into_spotify_param!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, usize, isize, bool, &str);
impl_into_spotify_param_with_ref!(String, Uri);

#[allow(dead_code)]
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
            format!("{}?{}", self.url, serde_urlencoded::to_string(self.params)?,)
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
    pub fn new<I: Into<flow::Config>>(
        credentials: F::Credentials,
        oauth: OAuth,
        config: I,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            api: F::setup(credentials, oauth, config.into())?,
        })
    }
}

pub fn validate_scope<'a, I: IntoIterator<Item = &'a str>>(scopes: &HashSet<String>, required: I) -> Result<(), Error> {
    let missing = required
        .into_iter()
        .filter(|scope| !scopes.contains(*scope))
        .map(|scope| scope.to_string())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(Error::ScopesNotGranted(missing));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, PartialOrd)]
pub enum UserResource {
    None,
    Collection,
    CollectionYourEpisodes,
}

impl Display for UserResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UserResource::None => "",
                UserResource::Collection => "collection",
                UserResource::CollectionYourEpisodes => "collection:your-episodes",
            }
        )
    }
}

impl FromStr for UserResource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "collection" => Ok(Self::Collection),
            "collection:your-episodes" => Ok(Self::CollectionYourEpisodes),
            "" => Ok(Self::None),
            _ => Err("Invalid spotify uri".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, PartialOrd)]
pub enum Resource {
    Artist,
    Album,
    Track,
    Playlist,
    User(UserResource),
    Show,
    Episode,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Resource::Album => "album",
                Resource::Artist => "artist",
                Resource::Track => "track",
                Resource::Playlist => "playlist",
                Resource::User(_) => "user",
                Resource::Show => "show",
                Resource::Episode => "episode",
            }
        )
    }
}

impl FromStr for Resource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "album" => Ok(Self::Album),
            "artist" => Ok(Self::Artist),
            "track" => Ok(Self::Track),
            "playlist" => Ok(Self::Playlist),
            "user" => Ok(Self::User(UserResource::Collection)),
            "show" => Ok(Self::Show),
            "episode" => Ok(Self::Episode),
            _ => Err("Invalid spotify uri".into()),
        }
    }
}

/// The resource identifier of, for example, an artist, album or track. This can be entered in the search box in a Spotify Desktop Client, to navigate to that resource. To find a Spotify URI, right-click (on Windows) or Ctrl-Click (on a Mac) on the artist, album or track name.
///
/// Example: spotify:track:6rqhFgbbKwnb9MLmUQDhG6
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Uri {
    resource: Resource,
    id: String,
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.resource {
            Resource::User(user_resource) => match user_resource {
                UserResource::None => write!(f, "spotify:user:{}", self.id),
                UserResource::Collection => write!(f, "spotify:user:{}:collection", self.id),
                UserResource::CollectionYourEpisodes => write!(f, "spotify:user:{}:collection:your-episodes", self.id),
            },
            other => write!(f, "spotify:{}:{}", other, self.id),
        }
    }
}

impl FromStr for Uri {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(3, ':').collect::<Vec<_>>();
        let resource = Resource::from_str(parts[1])?;
        match resource {
            Resource::User(_) => {
                if parts[2].contains(':') {
                    let (user_id, resource) = parts[2].split_once(':').unwrap();
                    Ok(Self {
                        resource: Resource::User(UserResource::from_str(resource)?),
                        id: user_id.to_string(),
                    })
                } else {
                    Ok(Self {
                        resource: Resource::User(UserResource::None),
                        id: parts[2].to_string(),
                    })
                }
            },
            other => Ok(Self {
                resource,
                id: parts[2].to_string(),
            })
        }
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

impl Uri {
    /// Id of the spotify uri
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Type of the spotify uri
    pub fn resource(&self) -> Resource {
        self.resource
    }

    pub fn artist<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::Artist,
            id: id.to_string(),
        }
    }

    pub fn album<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::Album,
            id: id.to_string(),
        }
    }

    pub fn track<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::Track,
            id: id.to_string(),
        }
    }

    pub fn playlist<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::Playlist,
            id: id.to_string(),
        }
    }

    pub fn user<S: Display>(id: S, resource: UserResource) -> Self {
        Uri {
            resource: Resource::User(resource),
            id: id.to_string(),
        }
    }

    pub fn collection<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::User(UserResource::Collection),
            id: id.to_string(),
        }
    }

    pub fn collection_your_episodes<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::User(UserResource::CollectionYourEpisodes),
            id: id.to_string(),
        }
    }

    pub fn show<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::Show,
            id: id.to_string(),
        }
    }

    pub fn episode<S: Display>(id: S) -> Self {
        Uri {
            resource: Resource::Episode,
            id: id.to_string(),
        }
    }
}

pub mod alphabet {
    pub static PKCE: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";
    pub static STATE: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-";
}

pub fn uuid<const N: usize>(alphabet: &[u8]) -> String {
    debug_assert!(N >= 43);
    debug_assert!(N <= 128);

    let mut buf = [0u8; N];
    getrandom::getrandom(&mut buf).unwrap();
    let range = alphabet.len();

    buf.iter()
        .map(|b| alphabet[*b as usize % range] as char)
        .collect()
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
