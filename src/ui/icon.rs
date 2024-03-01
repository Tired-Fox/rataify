use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaIcon {
    Pause,
    Play,
    Next,
    Previous,
    Shuffle,
    Repeat,
    RepeatOnce,
}