use std::path::PathBuf;
use lazy_static::lazy_static;
pub use keymap::KeyMap;

pub(crate) mod logging;
pub mod error;
mod keymap;
pub mod action;
pub mod config;
pub mod app;
mod event;
mod state;
pub mod ui;
pub mod spotify;

lazy_static! {
    #[cfg(windows)]
    pub static ref CONFIG_PATH: PathBuf = {
        #[cfg(windows)]
        return home::home_dir().unwrap().join(".rataify");
        #[cfg(not(windows))]
        return home::home_dir().unwrap().join(".config/rataify");
    };
}
