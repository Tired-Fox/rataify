use serde::{Deserialize, Serialize};

use crate::spotify::response::Playback;
pub use crate::{_private_action as private, _public_action as public};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublicAction {
    Next,
    Previous,
    NextTab,
    PreviousTab,
    Play,
    Pause,
    TogglePlayback,
    SelectDevice,
    Stop,
    VolumeUp,
    VolumeDown,
    ToggleShuffle,
    ToggleRepeat,
    Select,
    Down,
    Up,
    Left,
    Right,
    Help,
    Back,
    Exit,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PrivateAction {
    Tick,
    Render,
    FetchPlayback,
    Focus,
    Unfocus,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Action {
    Public(PublicAction),
    Private(PrivateAction),
    None,
}

impl From<Option<Action>> for Action {
    fn from(value: Option<Action>) -> Self {
        value.unwrap_or(Action::None)
    }
}

impl From<Public> for Action {
    fn from(value: Public) -> Self {
        Action::Public(value)
    }
}

impl From<Private> for Action {
    fn from(value: Private) -> Self {
        Action::Private(value)
    }
}

#[macro_export]
macro_rules! _public_action {
    ($action: ident) => {
        crate::action::Action::Public(crate::action::PublicAction::$action)
    };
}
#[macro_export]
macro_rules! _private_action {
    ($action: ident) => {
        crate::action::Action::Private(crate::action::PrivateAction::$action)
    };
}

pub type Private = PrivateAction;

pub type Public = PublicAction;

pub const NONE: Action = Action::None;
