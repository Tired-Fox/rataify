use std::{env, fmt::Display};

use base64::Engine;

/// User credentials to authenticate with the Spotify API.
///
/// Note: The Pkce flow does not require a client secret. All other flows require a client secret.
#[derive(Debug, Clone)]
pub struct Credentials { 
    pub(crate) id: String,
    pub(crate) secret: Option<String>,
}

impl Credentials {
    /// Create credentials from a client ID and a client secret
    pub fn new(client_id: &str, client_secret: Option<&str>) -> Self {
        Self {
            id: client_id.to_string(),
            secret: client_secret.map(|v| v.to_string()),
        }
    }

    /// Create credentials from environment variables
    ///
    /// # Variables
    /// - `TUPY_CLIENT_ID`: Client ID
    /// - `TUPY_CLIENT_SECRET`: Client secret
    pub fn from_env() -> Option<Self> {
        #[cfg(feature="env-file")]
        {
            dotenvy::dotenv().ok();
        }

        Some(Self::new(
            &env::var("TUPY_CLIENT_ID").ok()?,
            env::var("TUPY_CLIENT_SECRET").ok().as_deref()
        ))
    }
}

impl Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth = format!(
            "{}:{}",
            self.id,
            self.secret.as_ref().unwrap_or(&"".to_string())
        );
        
        write!(f, "{}", base64::engine::general_purpose::STANDARD.encode(auth.as_bytes()))
    }
}
