use crossterm::event::KeyEvent;
use rspotify::model::{Offset, PlayContextId};

use crate::state::window::modal::Modal;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Quit application
    Quit,
    /// Close current target, if the current target is a regular layout/window then quit
    Close,

    Up,
    Down,
    Left,
    Right,

    /// Next tab
    Tab,
    /// Previous tab
    BackTab,

    /// Select current selection
    Select,
    /// Next page in pagination
    NextPage,
    /// Previous page in pagination
    PreviousPage,

    /// Go to next item in the queue
    Next,
    /// Go to previous item in the queue
    Previous,
    /// Toggle playing state item in the queue
    Toggle,
    /// Toggle shuffle state
    Shuffle,
    /// Toggle to next repeat state
    Repeat,

    /// Open a modal
    Open(ModalOpen),
    /// Set the currently used device
    SetDevice(String, Option<bool>),

    /// Non mapped key
    Key(KeyEvent),

    Play(Play),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalOpen {
   Devices(Option<bool>)
}

impl ModalOpen {
   pub fn devices(play: Option<bool>) -> Self {
      Self::Devices(play)
   }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Play {
   Context(PlayContextId<'static>, Option<Offset>, Option<chrono::Duration>) 
}
