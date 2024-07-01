use std::sync::{Arc, Mutex};

pub mod tui;
pub mod errors;
pub mod app;
pub mod ui;
pub mod spotify_util;
pub mod state;

pub type Shared<T> = Arc<T>;
pub type Locked<T> = Mutex<T>;
