use std::collections::HashMap;
use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};

use crate::action::{Action, Public};
use crate::KeyMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Overrides {
    pub overwrite_duplicate_keymaps: bool,
}

#[derive(Default, Debug)]
pub struct Config {
    pub keymaps: HashMap<KeyEvent, Action>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ConfigBuilder {
    overrides: Option<Overrides>,
    keymaps: Option<HashMap<KeyMap, Public>>,
    #[serde(skip)]
    reserved: HashMap<KeyEvent, Action>,
}

impl Config {
    pub fn load_with_fallback<const N: usize>(paths: [&str; N]) -> Result<ConfigBuilder> {
        let files = paths
            .iter()
            .filter_map(|p| {
                let path = PathBuf::from(p);
                if !path.exists() {
                    return None;
                }
                Some(path)
            })
            .collect::<Vec<PathBuf>>();

        if files.is_empty() {
            Ok(ConfigBuilder::default())
        } else {
            let config_file = std::fs::read_to_string(files.first().unwrap())?;
            Ok(serde_yaml::from_str(config_file.as_str())?)
        }
    }
}

impl ConfigBuilder {
    pub fn reserved_keys(mut self, mappings: HashMap<KeyEvent, Action>) -> Self {
        self.reserved.extend(mappings);
        self
    }

    pub fn compile(self) -> Config {
        Config {
            keymaps: self.keymaps(),
        }
    }

    fn keymaps(&self) -> HashMap<KeyEvent, Action> {
        let mut keymaps = HashMap::new();

        if let Some(mappings) = &self.keymaps {
            for (key, action) in mappings.iter() {
                let event = KeyEvent::from(*key);

                // If keymapping exists and actions isn't the same, then skip because no overwrite
                if self.reserved.contains_key(&event) {
                    eprintln!("Reserved keymap `{key}`; skipping {key} -> {:?}", self.reserved.get(&event).unwrap());
                } else if keymaps.contains_key(&event) {
                    if &Action::from(*action) != keymaps.get(&event).unwrap() {
                        eprintln!(
                            "Duplicate keymap `{key}`: {key} -> {:?} conflicts with {key} -> {action:?}: Skipping",
                            keymaps.get(&event).unwrap(),
                        )
                    }
                } else {
                    keymaps.insert(event, Action::Public(*action));
                }
            }
        }

        keymaps.extend(self.reserved.clone());
        keymaps
    }
}

#[cfg(text)]
mod test {
    use super::*;

    #[test]
    fn test_keymap_parse() {

    }

    #[test]
    fn test_duplicate_keymap() {

    }

    #[test]
    fn test_reserved_keymap() {

    }
}