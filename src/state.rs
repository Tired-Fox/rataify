use std::{collections::HashMap, fmt::Debug};

use crossterm::event::KeyCode;
use ratatui::widgets::{ListState, TableState};
use tupy::{api::{response::{self, Device, PlaybackItem, Repeat}, Uri}, DateTime, Duration, Local};

use crate::{ui::action::{GoTo, UiAction}, Locked, Shared};

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

impl From<(response::Queue, Vec<bool>, Vec<bool>)> for Queue {
    fn from(mut q: (response::Queue, Vec<bool>, Vec<bool>)) -> Self {
        let mut saved_tracks = q.1.into_iter();
        let mut saved_episodes = q.2.into_iter();
        Self {
            items: q.0.queue.into_iter().map(|i| match &i {
                response::Item::Track(_) => Item::new(i, saved_tracks.next().unwrap_or(false)),
                response::Item::Episode(_) => Item::new(i, saved_episodes.next().unwrap_or(false)),
            }).collect(),
        }
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QueueState {
    pub state: TableState,
    pub queue: Loading<Queue>,
}

impl QueueState {
    pub fn next(&mut self) {
        if let Loading::Some(ref mut q) = self.queue {
            self.state.next_in_list(q.items.len());
        }
    }
    
    pub fn prev(&mut self) {
        if let Loading::Some(ref mut q) = self.queue {
            self.state.prev_in_list(q.items.len());
        }
    }

    pub fn select(&mut self) -> Option<Item> {
        if let Loading::Some(ref mut q) = self.queue {
            q.items.get(self.state.selected().unwrap_or(0)).cloned()
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WindowState {
    pub queue: QueueState,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DevicesState {
    pub state: TableState,
    pub devices: Vec<Device>,
}

impl DevicesState {
    pub fn next(&mut self) {
        self.state.next_in_list(self.devices.len());
    }
    
    pub fn prev(&mut self) {
        self.state.prev_in_list(self.devices.len());
    }

    pub fn select(&mut self) -> Device {
        self.devices[self.state.selected().unwrap_or(0)].clone()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GoToState {
    lookup: HashMap<KeyCode, usize>,
    pub mappings: Vec<(KeyCode, GoTo)>,
}

impl GoToState {
    pub fn new(mappings: Vec<(KeyCode, GoTo)>) -> Self {
        Self {
            lookup: mappings.iter().enumerate().map(|(i, (k, _))| (*k, i)).collect(),
            mappings,
        }
    }

    pub fn contains(&self, key: &KeyCode) -> bool {
        self.lookup.contains_key(key)
    }

    pub fn get(&self, key: &KeyCode) -> Option<&GoTo> {
        self.lookup.get(key).map(|i| &self.mappings[*i].1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModalState {
    pub devices: DevicesState,
    pub go_to: GoToState,
    pub actions: Vec<UiAction>,
    pub add_to_playlist: Option<Uri>,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            devices: DevicesState::default(),
            go_to: GoToState::new(vec![
                (KeyCode::Char('_'), GoTo::Queue),
                (KeyCode::Char('L'), GoTo::Library),
            ]),
            actions: vec![],
            add_to_playlist: None,
        }
    }
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
