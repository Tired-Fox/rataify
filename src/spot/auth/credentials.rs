use base64::Engine;
use serde::Deserialize;
use crate::CONFIG_PATH;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: Option<String>,
}

impl Credentials {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: Some(client_secret.to_string()),
        }
    }

    pub fn from_env() -> Option<Self> {
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(CONFIG_PATH.as_path()).ok()?;
        if dotenvy::dotenv().is_err() {
            return None;
        }
        let value = envy::prefixed("RATAIFY_").from_env();
        std::env::set_current_dir(old).ok()?;
        value.ok()
    }

    pub fn auth(&self) -> String {
        let auth = format!(
            "{}:{}",
            self.client_id,
            self.client_secret.as_ref().unwrap()
        );

        let base = base64::engine::general_purpose::STANDARD.encode(auth.as_bytes());
        base
    }
}
