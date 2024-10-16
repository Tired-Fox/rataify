use base64::Engine;
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::{api::{SpotifyResponse, uuid, alphabet}, Error};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::Path,
};

/// OAuth2 handler for scopes and redirect urls
#[derive(Debug, Clone)]
pub struct OAuth {
    pub redirect: String,
    pub state: String,
    pub scopes: HashSet<String>,
}

pub trait IntoScopes {
    fn into_scopes(self) -> HashSet<String>;
}

impl IntoScopes for () {
    fn into_scopes(self) -> HashSet<String> {
        HashSet::new()
    }
}

impl<A: AsRef<str>, const N: usize> IntoScopes for [A; N] {
    fn into_scopes(self) -> HashSet<String> {
        self.iter().map(|s| s.as_ref().to_string()).collect()
    }
}

impl<A: AsRef<str>> IntoScopes for &[A] {
    fn into_scopes(self) -> HashSet<String> {
        self.iter().map(|s| s.as_ref().to_string()).collect()
    }
}

impl<A: AsRef<str>> IntoScopes for HashSet<A> {
    fn into_scopes(self) -> HashSet<String> {
        self.iter().map(|s| s.as_ref().to_string()).collect()
    }
}

impl<A: AsRef<str>> IntoScopes for Vec<A> {
    fn into_scopes(self) -> HashSet<String> {
        self.iter().map(|s| s.as_ref().to_string()).collect()
    }
}

impl OAuth {
    pub fn new<S: IntoScopes>(redirect: String, scopes: S) -> Self {
        Self {
            redirect,
            state: uuid::<43>(alphabet::STATE),
            scopes: scopes.into_scopes(),
        }
    }

    pub fn from_env<S: IntoScopes>(scopes: S) -> Option<Self> {
        #[cfg(feature = "env-file")]
        {
            dotenvy::dotenv().ok();
        }

        Some(Self {
            redirect: std::env::var("TUPY_REDIRECT_URI").ok()?,
            state: uuid::<43>(alphabet::STATE),
            scopes: scopes.into_scopes(),
        })
    }
}

static DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

fn serialize_datetime<S>(datetime: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let datetime = datetime.format(DATETIME_FORMAT);
    serializer.serialize_str(datetime.to_string().as_str())
}

fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let scopes = String::deserialize(deserializer)?;
    let naive = NaiveDateTime::parse_from_str(scopes.as_str(), DATETIME_FORMAT)
        .map_err(serde::de::Error::custom)?;
    Local
        .from_local_datetime(&naive)
        .latest()
        .ok_or(serde::de::Error::custom("Invalid date"))
}

/// Spotify authentication token
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub scopes: HashSet<String>,
    pub refresh_token: Option<String>,
    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime"
    )]
    pub expires: DateTime<Local>,
}

impl Token {
    pub fn access(&self) -> &str {
        &self.access_token
    }
    pub fn ttype(&self) -> &str {
        &self.token_type
    }

    pub fn is_expired(&self) -> bool {
        self.expires <= Local::now()
    }

    pub fn save(&self, path: &Path, id: &str) -> Result<(), Error> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }

        let content = serde_json::to_string(self)?;
        let content = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
        std::fs::write(path.join(format!("spotify.{id}.token")), content)?;
        Ok(())
    }

    pub fn load(path: &Path, id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.join(format!("spotify.{id}.token")).exists() {
            return Err("No cached token found".into());
        }

        let body = std::fs::read_to_string(path.join(format!("spotify.{id}.token")))?;
        let body = String::from_utf8(base64::engine::general_purpose::STANDARD.decode(body)?)?;
        Ok(serde_json::from_str(&body)?)
    }

    pub fn parse_refresh<S: AsRef<str>>(&mut self, body: S) -> Result<(), Error> {
        let body: HashMap<String, serde_json::Value> = serde_json::from_str(body.as_ref())?;
        if body.contains_key("error_description") {
            return Err(Error::Auth {
                code: 400,
                error: body.get("error").unwrap().as_str().unwrap().to_string(),
                message: body.get("error_description").unwrap().as_str().unwrap().to_string(),
            })
        }

        self.access_token = body
            .get("access_token")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        // NOTE: Updating scopes may not be proper. Scopes should remain the same between refreshes
        //self.scopes = body
        //    .get("scope")
        //    .unwrap()
        //    .as_str()
        //    .unwrap()
        //    .split(' ')
        //    .map(|v| v.to_string())
        //    .collect();
        self.token_type = body
            .get("token_type")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        self.expires =
            Local::now() + Duration::seconds(body.get("expires_in").unwrap().as_i64().unwrap());
        if let Some(refresh_token) = body.get("refresh_token") {
            self.refresh_token = Some(refresh_token.as_str().unwrap().to_string());
        }
        Ok(())
    }

    pub fn from_auth(response: SpotifyResponse) -> Result<Self, Error> {
        let SpotifyResponse { body, .. } = response;
        let body: HashMap<String, serde_json::Value> = serde_json::from_str(&body)?;

        Ok(Self {
            access_token: body
                .get("access_token")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            token_type: body
                .get("token_type")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            scopes: body.get("scope").map_or(HashSet::new(), |v| {
                v.as_str()
                    .unwrap()
                    .split(' ')
                    .map(|v| v.to_string())
                    .collect()
            }),
            refresh_token: body
                .get("refresh_token")
                .map(|v| v.as_str().unwrap().to_string()),
            expires: body
                .get("expires_in")
                .map(|v| {
                    let seconds = v.as_i64().unwrap();
                    Local::now() + Duration::seconds(seconds)
                })
                .unwrap_or(Local::now()),
        })
    }
}
