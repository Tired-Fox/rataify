mod error;
mod playback;
mod queue;
mod user;

pub use error::*;
pub use playback::*;
pub use queue::*;
use serde::Deserialize;
pub use user::*;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
