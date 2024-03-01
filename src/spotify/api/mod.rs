pub mod body;
pub mod response;

use std::collections::HashMap;
use reqwest::header::{HeaderMap, IntoHeaderName};
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
pub use serde_json::json;
use serde_json::Value;
use crate::logging::ResponseLogger;
use crate::spotify::OAuth;

#[derive(Serialize, Deserialize)]
pub struct Form(HashMap<String, String>);

impl<const N: usize, S1: ToString, S2: ToString> From<[(S1, S2); N]> for Form {
    fn from(value: [(S1, S2); N]) -> Self {
        Self(HashMap::from_iter(value.iter().map(|(k, v)| (k.to_string(), v.to_string()))))
    }
}

pub enum Method {
    GET,
    POST,
    PUT,
}

/// Custom builder for reqwest::Client that abstracts the oauth token,
/// spotify api url, content length, and other features.
pub struct SpotifyRequest {
    method: Method,
    url: Option<String>,
    content_length: usize,
    params: HashMap<String, String>,
    headers: HeaderMap,
    body: Option<String>,
}

impl SpotifyRequest {
    pub fn new() -> Self {
        Self {
            method: Method::GET,
            content_length: 0,
            url: None,
            params: HashMap::new(),
            headers: HeaderMap::new(),
            body: None,
        }
    }

    pub fn get<S: ToString>(url: S) -> Self {
        Self::new().url(url)
    }

    pub fn post<S: ToString>(url: S) -> Self {
        Self::new().method(Method::POST).url(url)
    }

    pub fn put<S: ToString>(url: S) -> Self {
        Self::new().method(Method::PUT).url(url)
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn url<S: ToString>(mut self, url: S) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn param<S1: ToString, S2: ToString>(mut self, key: S1, value: S2) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn header<S: IntoHeaderName, T: ToString>(mut self, key: S, value: T) -> Self {
        self.headers.insert(key, value.to_string().parse().unwrap());
        self
    }

    /// Set the body of the request to json
    pub fn with_json<D: Serialize>(mut self, json: &D) -> Self {
        self.headers.insert("Content-Type", "application/json".parse().unwrap());
        let json = serde_json::to_string(json).unwrap();
        self.content_length = json.len();
        self.body = Some(json);
        self
    }

    /// Set the body of the request to an urlencoded form / query string
    pub fn with_form<'a, D: Deserialize<'a>>(mut self, form: Form) -> Self {
        self.headers.insert("Content-Type", "application/json".parse().unwrap());
        let form = serde_urlencoded::to_string(form).unwrap();
        self.content_length = form.len();
        self.body = Some(form);
        self
    }

    pub async fn send(self, oauth: &mut OAuth) -> color_eyre::Result<reqwest::Response> {
        oauth.update().await?;
        let client = reqwest::Client::new();
        let params = if self.params.is_empty() {
            String::new()
        } else {
            format!("?{}", self.params.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("&"))
        };
        let url = format!("https://api.spotify.com/v1{}{}", self.url.unwrap_or("me".to_string()), params);
        let content_length = self.content_length;
        let body = self.body;

        match self.method {
            Method::GET => {
                Ok(client.get(url)
                    .headers(self.headers)
                    .header("Authorization", oauth.token().unwrap().to_header())
                    .send()
                    .await?)
            }
            Method::POST => {
                let mut req = client.post(url)
                    .headers(self.headers)
                    .header("Content-Length", content_length)
                    .header("Authorization", oauth.token().unwrap().to_header());

                if let Some(body) = body {
                    req = req.body(body);
                }
                Ok(req.send().await?)
            }
            Method::PUT => {
                let mut req = client.put(url)
                    .headers(self.headers)
                    .header("Content-Length", content_length)
                    .header("Authorization", oauth.token().unwrap().to_header());

                if let Some(body) = body {
                    req = req.body(body);
                }
                Ok(req.send().await?)
            }
        }
    }
}

pub struct NoContent;

impl<'de> Deserialize<'de> for NoContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
    {
        let result = Value::deserialize(deserializer)?;
        match result {
            Value::Null => Ok(NoContent),
            Value::Object(map) => if map.len() == 0 {
                Ok(NoContent)
            } else {
                Err(serde::de::Error::custom("Expected no content"))
            },
            _ => Err(serde::de::Error::custom("Expected no content"))
        }
    }
}

pub enum SpotifyResponse<T>
    where
        T: Deserialize<'static>,
{
    Ok(T),
    NoDevice,
    ExpiredToken,
    Failed,
    ExceededRateLimit,
    Err(crate::error::Error),
}

impl<T> SpotifyResponse<T>
    where
        T: Deserialize<'static>,
{
    pub async fn from_response(response: color_eyre::Result<reqwest::Response>) -> Self {
        match response {
            Ok(response) => {
                let status = response.status().clone();
                match response.status() {
                    StatusCode::UNAUTHORIZED => SpotifyResponse::ExpiredToken,
                    StatusCode::NOT_FOUND => SpotifyResponse::NoDevice,
                    StatusCode::TOO_MANY_REQUESTS => SpotifyResponse::ExceededRateLimit,
                    StatusCode::OK | StatusCode::NO_CONTENT => match response.text().await {
                        Ok(mut ok) => {
                            if ok.is_empty() {
                                ok = String::from("null");
                            }

                            let jd = &mut serde_json::Deserializer::from_str(Box::leak(ok.into_boxed_str()));
                            match serde_path_to_error::deserialize(jd) {
                                Ok(ok) => SpotifyResponse::Ok(ok),
                                Err(err) => {
                                    ResponseLogger::log_error(status, &err.to_string()).await;
                                    SpotifyResponse::Err(crate::error::Error::custom(err))
                                }
                            }
                        }
                        Err(err) => {
                            ResponseLogger::log_error(status, &err.to_string()).await;
                            SpotifyResponse::Err(crate::error::Error::custom(err))
                        }
                    }
                    _ => {
                        let body = response.text().await.unwrap();
                        ResponseLogger::log_error(status, &body).await;
                        SpotifyResponse::Err(crate::error::Error::custom(body))
                    }
                }
            },
            Err(err) => {
                SpotifyResponse::Err(crate::error::Error::Custom(err))
            }
        }
    }
}