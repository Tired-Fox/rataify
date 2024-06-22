use std::{env, fmt::Display};

use base64::Engine;

#[derive(Debug, Clone)]
pub struct Credentials { 
    pub(crate) id: String,
    pub(crate) secret: String,
}

impl Credentials {
    /// Create credentials from a client ID and a client secret
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            id: client_id.to_string(),
            secret: client_secret.to_string(),
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
            &env::var("TUPY_CLIENT_SECRET").ok()?
        ))
    }
}

impl Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth = format!(
            "{}:{}",
            self.id,
            self.secret
        );
        
        write!(f, "{}", base64::engine::general_purpose::STANDARD.encode(auth.as_bytes()))
    }
}
