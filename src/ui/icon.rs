use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;

pub enum MediaIcon {
    Pause,
    Play,
    Next,
    Previous,
    Shuffle,
    Repeat,
    RepeatOnce,
}

impl From<MediaIcon> for Cow<'static, str> {
    fn from(value: MediaIcon) -> Self {
        match value {
            MediaIcon::Play => "▶".into(),
            MediaIcon::Pause => "⏸".into(),
            MediaIcon::Next => "⏭".into(),
            MediaIcon::Previous => "⏮".into(),
            MediaIcon::Shuffle => "🔀".into(),
            MediaIcon::Repeat => "🔁".into(),
            MediaIcon::RepeatOnce => "🔂".into(),
        }
    }
}

impl Display for MediaIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MediaIcon::Pause => "⏸",
            MediaIcon::Play => "▶",
            MediaIcon::Next => "⏭",
            MediaIcon::Previous => "⏮",
            MediaIcon::Shuffle => "🔀",
            MediaIcon::Repeat => "🔁",
            MediaIcon::RepeatOnce => "🔂",

        })
    }
}

impl MediaIcon {
    pub fn play_pause(state: bool) -> Self {
        if state {
            Self::Play
        } else {
            Self::Pause
        }
    }
}