use crossterm::event::{KeyCode, KeyModifiers, MediaKeyCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    modifiers: KeyModifiers,
    key: KeyCode,
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut parts = Vec::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
           parts.push("ctrl".to_string())
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
           parts.push("alt".to_string())
        }
        parts.push(self.key.to_string().to_ascii_lowercase());
        
        serializer.serialize_str(parts.join("+").as_str())
    }
}

impl<'de> Deserialize<'de> for Key {
   fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
       where
           D: serde::Deserializer<'de> {
       let key = String::deserialize(deserializer)?;

       let mut modifiers = KeyModifiers::empty();
       let parts = key.split("+").collect::<Vec<_>>();
      for part in &parts[..parts.len()-1] {
         match part.to_ascii_lowercase().as_str() {
            "ctrl" => modifiers.insert(KeyModifiers::CONTROL),
            "alt" => modifiers.insert(KeyModifiers::CONTROL),
            "shift" => {},
            other => return Err(serde::de::Error::custom(format!("unknown key modifier {other}")))
         }
      }

      let last = parts.last().unwrap();
      let first = last.chars().next().unwrap();
      if last.len() == 1 {
         return Ok(Key { modifiers, key: KeyCode::Char(first) });
      }

      if (first == 'F' || first == 'f') && last[1..].chars().all(|c| c.is_ascii_digit()) {
        return Ok(Key { key: KeyCode::F((&last[1..]).parse::<u8>().map_err(serde::de::Error::custom)?), modifiers })
      }

      let keycode = match last.to_ascii_lowercase().as_str() {
         "backspace" => KeyCode::Backspace,
         "enter" => KeyCode::Enter,
         "left" => KeyCode::Left,
         "right" => KeyCode::Right,
         "up" => KeyCode::Up,
         "down" => KeyCode::Down,
         "home" => KeyCode::Home,
         "end" => KeyCode::End,
         "pageup" => KeyCode::PageUp,
         "pagedown" => KeyCode::PageDown,
         "tab" => KeyCode::Tab,
         "backtab" => KeyCode::BackTab,
         "delete" => KeyCode::Delete,
         "insert" => KeyCode::Insert,
         "null" => KeyCode::Null,
         "esc" => KeyCode::Esc,
         "capslock" => KeyCode::CapsLock,
         "scrolllock" => KeyCode::ScrollLock,
         "numlock" => KeyCode::NumLock,
         "printscreen" => KeyCode::PrintScreen,
         "menu" => KeyCode::Menu,
         "keypadbegin" => KeyCode::KeypadBegin,
         "play" => KeyCode::Media(MediaKeyCode::Play),
         "pause" => KeyCode::Media(MediaKeyCode::Pause),
         "playpause" => KeyCode::Media(MediaKeyCode::PlayPause),
         "reverse" => KeyCode::Media(MediaKeyCode::Reverse),
         "stop" => KeyCode::Media(MediaKeyCode::Stop),
         "fastforward" => KeyCode::Media(MediaKeyCode::FastForward),
         "rewind" => KeyCode::Media(MediaKeyCode::Rewind),
         "tracknext" => KeyCode::Media(MediaKeyCode::TrackNext),
         "trackprevious" => KeyCode::Media(MediaKeyCode::TrackPrevious),
         "record" => KeyCode::Media(MediaKeyCode::Record),
         "lowervolume" => KeyCode::Media(MediaKeyCode::LowerVolume),
         "raisevolume" => KeyCode::Media(MediaKeyCode::RaiseVolume),
         "mutevolume" => KeyCode::Media(MediaKeyCode::MuteVolume),
         other => return Err(serde::de::Error::custom(format!("unknown key for mapping: {other}")))
      };

      Ok(Key {
          modifiers,
          key: keycode
      })
   }
}
