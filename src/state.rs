use std::fmt::Debug;

use ratatui::widgets::ListState;
use tupy::{api::response::{self, Device, PlaybackItem, Repeat}, DateTime, Duration, Local};

use crate::{Locked, Shared};

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
pub struct Playback {
    pub device: Option<Device>,
    pub repeat: Repeat,
    pub shuffle: bool,
    pub progress: Option<Duration>,
    pub is_playing: bool,
    pub item: PlaybackItem,
}

impl Playback {
    pub fn set_repeat(&mut self, repeat: Repeat) {
        self.repeat = repeat;
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        self.shuffle = shuffle;
    }

    pub fn set_progress(&mut self, progress: Option<Duration>) {
        self.progress = progress;
    }

    pub fn set_playing(&mut self, is_playing: bool) {
        self.is_playing = is_playing;
    }
}

impl From<response::Playback> for Playback {
    fn from(pb: response::Playback) -> Self {
        Self {
            device: pb.device,
            repeat: pb.repeat,
            shuffle: pb.shuffle,
            progress: pb.progress,
            is_playing: pb.is_playing,
            item: pb.item,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub item: response::Item,
    pub saved: bool,
}

impl Item {
    pub fn new(item: response::Item, saved: bool) -> Self {
        Self {
            item,
            saved,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Queue {
    pub items: Vec<Item>,
}

impl From<(response::Queue, Vec<bool>)> for Queue {
    fn from(mut q: (response::Queue, Vec<bool>)) -> Self {
        Self {
            items: q.0.queue.into_iter().map(|i| match &i {
                response::Item::Track(_) => Item::new(i, q.1.pop().unwrap_or(false)),
                response::Item::Episode(_) => Item::new(i, false),
            }).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    Devices,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Window {
    #[default]
    Queue,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Viewport {
    Modal(Modal),
    #[default]
    Window,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QueueState {
    pub state: ListState,
    pub queue: Loading<Queue>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WindowState {
    pub queue: QueueState,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DevicesState {
    pub state: ListState,
    pub devices: Vec<Device>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModalState {
    pub devices: DevicesState,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub viewport: Viewport,
    pub window: Window,

    pub modal_state: Shared<Locked<ModalState>>,
    pub window_state: Shared<Locked<WindowState>>,

    pub last_playback_poll: Shared<Locked<DateTime<Local>>>,
    pub playback_poll: Countdown,
    pub playback: Shared<Locked<Option<Playback>>>,
}

impl State {
    pub fn show_queue(&self) -> bool {
        #[allow(irrefutable_let_patterns)]
        if let Window::Queue = self.window {
            return true;
        }
        false
    }
}
