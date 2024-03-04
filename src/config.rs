use std::collections::HashMap;
use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};

use crate::action::{Action, Public};
use crate::{CONFIG_PATH, KeyMap};

fn true_default() -> bool {
    true
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Overrides {
    pub overwrite_duplicate_keymaps: bool,
}

#[derive(Default, Debug)]
pub struct Config {
    pub keymaps: HashMap<KeyEvent, Action>,
}

#[derive(Default, Deserialize)]
pub struct ConfigBuilder {
    keymaps: Option<HashMap<KeyMap, Public>>,
    #[serde(skip)]
    reserved_keymaps: HashMap<KeyEvent, Action>,
    #[serde(skip)]
    default_keymaps: HashMap<KeyEvent, Action>,
}

impl Config {
    /// Load config from file with fallback file paths
    pub fn load_with_fallback<const N: usize>(paths: [&str; N]) -> Result<ConfigBuilder> {
        let files = paths
            .iter()
            .filter_map(|p| {
                let path = CONFIG_PATH.join(p);
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
    /// Set keybindings that cannot be overridden
    pub fn reserved_keys(mut self, mappings: HashMap<KeyEvent, Action>) -> Self {
        self.reserved_keymaps.extend(mappings);
        self
    }

    // Set default keymaps that can be overridden with new keybindings
    pub fn default_keys(mut self, mappings: HashMap<KeyEvent, Action>) -> Self {
        self.default_keymaps.extend(mappings);
        self
    }

    pub fn compile(mut self) -> Config {
        Config {
            keymaps: self.keymaps(),
        }
    }

    fn keymaps(&self) -> HashMap<KeyEvent, Action> {
        let mut keymaps = self.default_keymaps.clone();

        if let Some(mappings) = &self.keymaps {
            for (key, action) in mappings.iter() {
                let event = KeyEvent::from(*key);

                // If keymapping exists and actions isn't the same, then skip because no overwrite
                if self.reserved_keymaps.contains_key(&event) {
                    eprintln!(
                        "Reserved keymap `{key}`; skipping {key} -> {:?}",
                        self.reserved_keymaps.get(&event).unwrap()
                    );
                } else {
                    keymaps.insert(event, Action::Public(*action));
                }
            }
        }

        keymaps.extend(self.reserved_keymaps.clone());
        keymaps
    }
}

