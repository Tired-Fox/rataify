mod error;
mod playback;
mod user;

use serde::Deserialize;
pub use error::*;
pub use playback::*;
pub use user::*;

#[derive(Debug, Deserialize)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,

}
