use std::collections::HashMap;
use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};

use crate::action::{Action, Public};
use crate::KeyMap;
use crate::ui::icon::MediaIcon;

fn true_default() -> bool { true }

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Overrides {
    pub overwrite_duplicate_keymaps: bool,
}

#[derive(Default, Debug)]
pub struct Config {
    pub icons: IconsConfig,
    pub keymaps: HashMap<KeyEvent, Action>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IconsConfig {
    #[serde(default = "true_default")]
    pub on: bool,
    pub custom: HashMap<MediaIcon, String>,
}

impl IconsConfig {
    pub fn shuffle(&self) -> &str {
        self.custom.get(&MediaIcon::Shuffle).unwrap()
    }

    pub fn repeat(&self, repeat: crate::spotify::response::Repeat) -> &str {
        match repeat {
            crate::spotify::response::Repeat::Track => self.custom.get(&MediaIcon::Repeat).unwrap(),
            crate::spotify::response::Repeat::Context => self.custom.get(&MediaIcon::RepeatOnce).unwrap(),
            crate::spotify::response::Repeat::Off => self.custom.get(&MediaIcon::Repeat).unwrap(),
        }
    }

    pub fn pause(&self) -> &str {
        self.custom.get(&MediaIcon::Pause).unwrap()
    }

    pub fn play(&self) -> &str {
        self.custom.get(&MediaIcon::Play).unwrap()
    }

    pub fn next(&self) -> &str {
        self.custom.get(&MediaIcon::Next).unwrap()
    }

    pub fn previous(&self) -> &str {
        self.custom.get(&MediaIcon::Previous).unwrap()
    }
}

impl Default for IconsConfig {
    fn default() -> Self {
        Self {
            on: true,
            custom: HashMap::new(),
        }
    }
}

#[derive(Default, Deserialize)]
pub struct ConfigBuilder {
    #[serde(default)]
    icons: IconsConfig,
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

    pub fn compile(mut self) -> Config {
        let mut default_icons = HashMap::from([
            (MediaIcon::Pause, "▶".to_string()),
            (MediaIcon::Play, "⏸".to_string()),
            (MediaIcon::Next, "⏭".to_string()),
            (MediaIcon::Previous, "⏮".to_string()),
            (MediaIcon::Shuffle, "🔀".to_string()),
            (MediaIcon::Repeat, "🔁".to_string()),
            (MediaIcon::RepeatOnce, "🔂".to_string()),
        ]);

        for (key, icon) in default_icons.iter() {
            if !self.icons.custom.contains_key(key) {
                self.icons.custom.insert(*key, icon.to_string());
            }
        }

        Config {
            keymaps: self.keymaps(),
            icons: self.icons,
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