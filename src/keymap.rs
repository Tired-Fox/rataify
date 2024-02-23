use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

use color_eyre::Report;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd)]
pub struct KeyMap {
    pub modifiers: KeyModifiers,
    pub code: KeyCode
}

impl From<&str> for KeyMap {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap()
    }
}

impl From<KeyMap> for KeyEvent {
    fn from(value: KeyMap) -> Self {
        Self::new(value.code, value.modifiers)
    }
}

impl From<KeyEvent> for KeyMap {
    fn from(value: KeyEvent) -> Self {
        Self {
            modifiers: value.modifiers,
            code: value.code
        }
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            modifiers: KeyModifiers::empty(),
            code: KeyCode::Null
        }
    }
}

impl<'de> Deserialize<'de> for KeyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct Mapping;
        impl<'de> Visitor<'de> for Mapping {
            type Value = KeyMap;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
                Ok(FromStr::from_str(v).unwrap())
            }
        }

        deserializer.deserialize_str(Mapping)
    }
}


impl FromStr for KeyMap {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.replace(" ", "");
        let parts = parts.split("+");

        let mut keymap = Self::default();

        for part in parts {
            match part.to_ascii_lowercase().as_str() {
                ""  => {
                    if keymap.code == KeyCode::Null {
                        keymap.code = KeyCode::Char('+');
                    }
                },
                "ctrl" => keymap.modifiers |= KeyModifiers::CONTROL,
                "shift" => keymap.modifiers |= KeyModifiers::SHIFT,
                "alt" => keymap.modifiers |= KeyModifiers::ALT,
                value => {
                    if let KeyCode::Null = keymap.code {
                        match value {
                            "backspace" => keymap.code = KeyCode::Backspace,
                            "enter" => keymap.code = KeyCode::Enter,
                            "space" => keymap.code = KeyCode::Char(' '),
                            "up" => keymap.code = KeyCode::Up,
                            "down" => keymap.code = KeyCode::Down,
                            "left" => keymap.code = KeyCode::Left,
                            "right" => keymap.code = KeyCode::Right,
                            "home" => keymap.code = KeyCode::Home,
                            "end" => keymap.code = KeyCode::End,
                            "pageup" => keymap.code = KeyCode::PageUp,
                            "pagedown" => keymap.code = KeyCode::PageDown,
                            "insert" => keymap.code = KeyCode::Insert,
                            "delete" => keymap.code = KeyCode::Delete,
                            "f1" => keymap.code = KeyCode::F(1),
                            "f2" => keymap.code = KeyCode::F(2),
                            "f3" => keymap.code = KeyCode::F(3),
                            "f4" => keymap.code = KeyCode::F(4),
                            "f5" => keymap.code = KeyCode::F(5),
                            "f6" => keymap.code = KeyCode::F(6),
                            "f7" => keymap.code = KeyCode::F(7),
                            "f8" => keymap.code = KeyCode::F(8),
                            "f9" => keymap.code = KeyCode::F(9),
                            "f10" => keymap.code = KeyCode::F(10),
                            "f11" => keymap.code = KeyCode::F(11),
                            "f12" => keymap.code = KeyCode::F(12),
                            "esc" => keymap.code = KeyCode::Esc,
                            "tab" => keymap.code = KeyCode::Tab,
                            "print_screen" => keymap.code = KeyCode::PrintScreen,
                            "scroll_lock" => keymap.code = KeyCode::PrintScreen,
                            value if value.len() == 1 && value.is_ascii() => {
                                let value = part.chars().next().unwrap();
                                if value.is_ascii_uppercase() {
                                    keymap.modifiers |= KeyModifiers::SHIFT;
                                }
                                keymap.code = KeyCode::Char(value.to_ascii_lowercase());
                            }
                            _ => {
                                return Err(Report::msg(format!("Unknown key: {value}")));
                            }
                        }
                    } else {
                        return Err(Report::msg(format!("Unknown key: {value}")));
                    }
                }
            }
        }

        if keymap.code == KeyCode::Null {
            return Err(Report::msg("Must provide a key for a keymap"));
        }

        if let KeyCode::Char(value) = keymap.code {
            if !value.is_ascii_digit() && !value.is_alphabetic() {
                keymap.modifiers &= !KeyModifiers::SHIFT;
            }
        }

        Ok(keymap)
    }
}

impl Display for KeyMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if self.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
            parts.push("ctrl".to_string());
        }
        if self.modifiers & KeyModifiers::ALT == KeyModifiers::ALT {
            parts.push("alt".to_string());
        }
        if self.modifiers & KeyModifiers::SHIFT == KeyModifiers::SHIFT {
            parts.push("shift".to_string());
        }

        parts.push(match self.code {
            KeyCode::Char(value) => value.to_ascii_lowercase().to_string(),
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Up => "up".to_string(),
            KeyCode::Down => "down".to_string(),
            KeyCode::Left => "left".to_string(),
            KeyCode::Right => "right".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pageup".to_string(),
            KeyCode::PageDown => "pagedown".to_string(),
            KeyCode::Insert => "insert".to_string(),
            KeyCode::Delete => "delete".to_string(),
            KeyCode::F(1) => "f1".to_string(),
            KeyCode::F(2) => "f2".to_string(),
            KeyCode::F(3) => "f3".to_string(),
            KeyCode::F(4) => "f4".to_string(),
            KeyCode::F(5) => "f5".to_string(),
            KeyCode::F(6) => "f6".to_string(),
            KeyCode::F(7) => "f7".to_string(),
            KeyCode::F(8) => "f8".to_string(),
            KeyCode::F(9) => "f9".to_string(),
            KeyCode::F(10) => "f10".to_string(),
            KeyCode::F(11) => "f11".to_string(),
            KeyCode::F(12) => "f12".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Tab => "tab".to_string(),
            _ => "".to_string(),
        });

        write!(f, "{}", parts.join("+"))
    }
}

impl Serialize for KeyMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[macro_export]
macro_rules! keymaps {
    ($($key: literal => $value: expr),* $(,)?) => {
        {
            use std::str::FromStr;

            let mut mappings = std::collections::HashMap::new();
            $(
                mappings.insert(
                    crossterm::event::KeyEvent::from($crate::KeyMap::from_str($key).unwrap()),
                    $crate::action::Action::from($value)
                );
            )*
            mappings
        }
    };
    (@ Public($action: ident)) => {
        $crate::action::Action::Public($crate::action::Public::$action)
    };
    (@ Private($action: ident)) => {
        $crate::action::Action::Private($crate::action::Private::$action)
    };
    (@ None) => {
        $crate::action::Action::None
    };
}

pub use crate::keymaps;