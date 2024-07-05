use std::fmt::Debug;
use color_eyre::Result;

use ratatui::widgets::{ListState, TableState};
use modal::ModalState;
use tupy::api::flow::Pkce;
use window::WindowState;

use crate::{Locked, Shared};

use self::playback::Playback;

pub mod modal;
pub mod window;
pub mod playback;

pub trait IterCollection {
    fn next_in_list(&mut self, len: usize);
    fn prev_in_list(&mut self, len: usize);
}

impl IterCollection for ListState {
    fn next_in_list(&mut self, len: usize) {
        match self.selected() {
            Some(selected) if selected < len - 1 => {
                self.select(Some(selected + 1));
            }
            None => {
                self.select(Some(0));
            }
            _ => {}
        }
    }
    
    fn prev_in_list(&mut self, len: usize) {
        match self.selected() {
            Some(selected) if selected > 0 => {
                self.select(Some(selected - 1));
            }
            None => {
                self.select(Some(len - 1));
            }
            _ => {}
        }
    }
}

impl IterCollection for TableState {
    fn next_in_list(&mut self, len: usize) {
        match self.selected() {
            Some(selected) if selected < len - 1 => {
                self.select(Some(selected + 1));
            }
            None => {
                self.select(Some(0));
            }
            _ => {}
        }
    }
    
    fn prev_in_list(&mut self, len: usize) {
        match self.selected() {
            Some(selected) if selected > 0 => {
                self.select(Some(selected - 1));
            }
            None => {
                self.select(Some(len - 1));
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub enum Loading<T> {
    #[default]
    Loading,
    None,
    Some(T)
}

impl<T> Loading<T> {
    #[inline]
    pub const fn is_some(&self) -> bool {
        matches!(*self, Loading::Some(_))
    }

    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(*self, Loading::None)
    }

    #[inline]
    pub const fn is_loading(&self) -> bool {
        matches!(*self, Loading::Loading)
    }

    #[inline]
    pub const fn as_ref(&self) -> Loading<&T> {
        match *self {
            Self::Some(ref t) => Loading::Some(t),
            Self::None => Loading::None,
            Self::Loading => Loading::Loading,
        }
    }
}

impl<T> From<Option<T>> for Loading<T> {
    fn from(o: Option<T>) -> Self {
        match o {
            Some(t) => Self::Some(t),
            None => Self::None,
        }
    }
}

impl<T: Clone> Clone for Loading<T> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Loading => Self::Loading,
            Self::Some(t) => Self::Some(t.clone()),
        }
    }
}

impl<T: Debug> Debug for Loading<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Loading => write!(f, "Loading"),
            Self::Some(t) => write!(f, "Some({:?})", t),
        }
    }
}

impl<T: PartialEq> PartialEq for Loading<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Loading, Self::Loading) => true,
            (Self::Some(t), Self::Some(o)) => t == o,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Countdown {
    count: usize,
    _origin: usize,
}
impl Countdown {
    pub fn new(origin: usize) -> Self {
        Self {
            count: origin,
            _origin: origin,
        }
    }

    pub fn decrement(&mut self) {
        self.count = self.count.saturating_sub(1);
    }
    pub fn poll(&mut self) -> bool {
        self.decrement();
        self.is_ready()
    }
    pub fn is_ready(&self) -> bool {
        self.count == 0
    }
    pub fn reset(&mut self) {
        self.count = self._origin;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    Devices,
    Action,
    AddToPlaylist,
    GoTo,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Window {
    Queue,
    #[default]
    Library,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Viewport {
    Modal(Modal),
    #[default]
    Window,
}

#[derive(Debug, Clone)]
pub struct State {
    // State for what is shown in the viewport
    pub viewport: Viewport,
    pub window: Window,

    // State for the current modal
    pub modal_state: ModalState,
    // State for the current window
    pub window_state: WindowState,

    // Countdown for when to poll for playback
    pub playback_poll: Countdown,
    pub playback: Shared<Locked<playback::PlaybackState>>,
}

impl State {
    pub async fn new(dir: &str, api: &Pkce, countdown: Countdown, playback: Option<Playback>) -> Result<Self> {
        Ok(Self {
            viewport: Viewport::default(),
            window: Window::default(),

            modal_state: ModalState::default(),
            window_state: WindowState::new(dir, api).await?,

            playback_poll: countdown,
            playback: Shared::new(Locked::new(playback::PlaybackState::new(playback))),
        })
    }

    pub fn show_queue(&self) -> bool {
        #[allow(irrefutable_let_patterns)]
        if let Window::Queue = self.window {
            return true;
        }
        false
    }
}
