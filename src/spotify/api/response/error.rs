use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Error {
    pub status: u32,
    pub message: String,
    pub reason: Option<String>
}