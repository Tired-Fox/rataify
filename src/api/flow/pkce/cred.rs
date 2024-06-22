use std::env;

use base64::Engine;
use sha2::{Digest, Sha256};


static PKCE_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";

fn verifier<const N: usize>(alphabet: &[u8]) -> String {
    debug_assert!(N >= 43);
    debug_assert!(N <= 128);

    let mut buf = [0u8; N];
    getrandom::getrandom(&mut buf).unwrap();
    let range = alphabet.len();

    buf.iter()
        .map(|b| alphabet[*b as usize % range] as char)
        .collect()
}

fn sha256<S: AsRef<[u8]>>(value: S) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value);
    hasher.finalize().to_vec()
}

fn base64encode<S: AsRef<[u8]>>(value: S) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value)
}

#[derive(Debug, Clone)]
pub struct Credentials { 
    pub(crate) id: String,
    pub(crate) verifier: String,
    pub(crate) challenge: String
}

impl Credentials {
    /// Create credentials from a client ID
    ///
    /// <N> Length of the pkce verifier. The length must be between 48 and 128 characters.
    pub fn new(client_id: &str) -> Self {
        let verifier = verifier::<43>(PKCE_ALPHABET);
        let challenge = base64encode(sha256(&verifier));

        Self {
            id: client_id.to_string(),
            verifier,
            challenge
        }
    }

    /// Create credentials from environment variables
    ///
    /// <N> Length of the pkce verifier. The length must be between 48 and 128 characters.
    /// 
    /// # Variables
    /// - `TUPY_CLIENT_ID`: Client ID
    pub fn from_env() -> Option<Self> {
        #[cfg(feature="env-file")]
        {
            dotenvy::dotenv().ok();
        }

        Some(Self::new(&env::var("TUPY_CLIENT_ID").ok()?))
    }
}
