use std::{collections::{hash_map::Iter, HashMap}, iter::Map};

use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};

use crate::{action::{Action, Open}, input::Key, Error, key};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    keymaps: HashMap<Key, Action>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keymaps: HashMap::from([
                (key!('q'), Action::Close),
                (key!('g'), Action::Open(Open::GoTo)),
                (key!(Tab), Action::Tab),
                (key!(BackTab), Action::BackTab),
                (key!('h'), Action::Left),
                (key!('j'), Action::Down),
                (key!('k'), Action::Up),
                (key!('l'), Action::Right),
                (key!(Up), Action::Up),
                (key!(Down), Action::Down),
                (key!(Left), Action::Left),
                (key!(Right), Action::Right),
                (key!(Enter), Action::Select),
                (key!('<'), Action::PreviousPage),
                (key!('>'), Action::NextPage),
                (key!(' '), Action::Toggle),
                (key!('n'), Action::Next),
                (key!('p'), Action::Previous),
                (key!('s'), Action::Shuffle),
                (key!('r'), Action::Repeat),
                (key!('d'), Action::Open(Open::devices(None))),
            ]),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let config_dir = dirs::config_dir()
            .unwrap()
            .join("rataify")
            .join("config.json");
        let config = if config_dir.exists() {
            let cfg = std::fs::read_to_string(config_dir)?;
            serde_json::from_str(cfg.as_str())?
        } else {
            let config_dir = dirs::home_dir().unwrap().join(".rataify.json");
            if config_dir.exists() {
                let cfg = std::fs::read_to_string(config_dir)?;
                serde_json::from_str(cfg.as_str())?
            } else {
                Config::default()
            }
        };

        let mut default = Config::default();
        default.keymaps.extend(config.keymaps);
        Ok(default)
    }

    pub fn keymaps(&self) -> impl Iterator<Item = (KeyEvent, Action)> + '_ {
        self.keymaps
            .iter()
            .map(|(k, v)| (KeyEvent::from(*k), v.clone()))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use rspotify::model::PlaylistId;

    use crate::{
        action::{Action, Play},
        config::Config,
        input,
    };

    #[test]
    fn serialize() {
        let config = Config::load();
        println!("{:#?}", config);
    }
}
