use std::collections::HashMap;

use crossterm::event::KeyEvent;
use serde::Deserialize;

use crate::action::Action;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
   keymaps: HashMap<KeyEvent, Action>
}
