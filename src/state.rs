use std::fmt::{Debug, Display};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use std::time::Instant;

use chrono::Duration;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::action::Action;
use crate::config::IconsConfig;
use crate::spotify::response::{Album, Device, Episode, Item, Playback, Repeat, Show, Track};

lazy_static::lazy_static! {
    // TODO: Change to proper background algroithms instead
    static ref PATTERNS: [&'static str; 4] = [
        "…. ",
        "░▒▓█ ",
        "▗▖▝▘▚▞▙▛▜▟ ",
        "◢◣◥◤ ",
        // "▁▂▃▄▅▆▇█▋▌▍▎▏",
    ];
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
struct Flag(u8);

impl BitAnd for Flag {
    type Output = Flag;
    fn bitand(self, rhs: Self) -> Self::Output {
        Flag(self.0 & rhs.0)
    }
}

impl BitOr for Flag {
    type Output = Flag;
    fn bitor(self, rhs: Self) -> Self::Output {
        Flag(self.0 | rhs.0)
    }
}

impl BitOrAssign for Flag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAndAssign for Flag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

const TLBR: Flag = Flag(1);
const TRBL: Flag = Flag(2);

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub cover: Vec<Vec<char>>,

    pub current: Option<Playback>,
    pub last_polled: Instant,

    corners: Flag, // 1 = tl + br, 2 = tr + bl
}

impl Default for PlaybackState {
    fn default() -> Self {
        let mut state = Self {
            cover: Vec::new(),
            current: None,
            last_polled: Instant::now(),
            corners: Flag::default(),
        };
        state.generate_cover();
        state
    }
}

impl PlaybackState {
    pub fn shuffle(&self) -> bool {
        if let Some(playback) = &self.current {
            return playback.shuffle;
        }
        false
    }

    pub fn repeat(&self) -> Repeat {
        if let Some(playback) = &self.current {
            return playback.repeat;
        }
        Repeat::Off
    }

    /// Current state of whether something is playing
    pub fn playing(&self) -> bool {
        if let Some(playback) = &self.current {
            return playback.playing;
        }
        false
    }

    /// Get the name of the context, this could be an album, playlist, show, etc...
    pub fn context_name(&mut self) -> String {
        match &self.current {
            Some(playback) => {
               match &playback.item {
                   Some(Item::Track(Track { album: Album { name, ..}, .. })) => {
                       name.clone()
                   },
                   Some(Item::Episode(Episode { show: Some(Show { name, ..}), ..})) => {
                       name.clone()
                   },
                   _ => String::new()
               }
            },
            None => String::new()
        }
    }

    pub fn now_playing(&mut self, playback: Option<Playback>)
    {
        self.corners = Flag::default();
        self.current = playback;
        self.last_polled = Instant::now();

        self.generate_cover();
    }

    pub fn name(&self) -> String {
        if let Some(playback) = &self.current {
            match &playback.item {
                Some(Item::Track(track)) => track.name.clone(),
                Some(Item::Episode(episode)) => episode.name.clone(),
                _ => String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn artists(&self) -> Vec<String> {
        if let Some(playback) = &self.current {
            match &playback.item {
                Some(Item::Track(track)) => track.artists.iter().map(|a| a.name.clone()).collect(),
                Some(Item::Episode(episode)) => episode.show.as_ref().map_or(vec![], |e| vec![e.name.clone()]),
                _ => Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Percent completed
    pub fn percent(&self) -> f64 {
        let duration = self.duration().num_milliseconds();
        let progress = self.progress().num_milliseconds();

        if progress == 0 || duration == 0 {
            return 0.0;
        } else {
            progress as f64 / duration as f64
        }
    }

    pub fn elapsed(&self) -> Duration {
        Duration::milliseconds(self.last_polled.elapsed().as_millis() as i64)
    }

    pub fn progress(&self) -> Duration {
        match &self.current {
            Some(Playback { progress, .. }) => match self.playing() {
                true => (progress.unwrap_or(Duration::zero()).clone() + self.elapsed()).min(self.duration()),
                false => progress.unwrap_or(Duration::zero()).clone()
            },
            _ => Duration::zero()
        }
    }

    pub fn duration(&self) -> Duration {
        match &self.current {
            Some(Playback { item: Some(Item::Track(Track { duration, .. })), .. }) => duration.clone(),
            Some(Playback { item: Some(Item::Episode(Episode { duration, .. })), .. }) => duration.clone(),
            _ => Duration::zero()
        }
    }

    fn generate_cover(&mut self) {
        if self.current.is_none() {
            self.cover = (0..50).map(|_| (0..50).map(|_| ' ').collect()).collect();
            return
        }

        let mut hasher = DefaultHasher::default();
        match &self.current {
            Some(Playback { item: Some(Item::Track(track)), .. }) => {
                track.album.name.clone()
            }
            Some(Playback { item: Some(Item::Episode(episode)), .. }) => {
                match &episode.show {
                    Some(show) => {
                        show.name.clone()
                    }
                    None => episode.name.clone()
                }
            }
            _ => String::new()
        }.hash(&mut hasher);

        let mut rng = StdRng::seed_from_u64(hasher.finish());
        let pattern: usize = rng.gen_range(0..PATTERNS.len());
        let mut pattern = PATTERNS[pattern].chars().collect::<Vec<char>>();

        let scale = rng.gen_range(pattern.len()..pattern.len() * 12);
        // Pick random characters from pattern
        let picks = rng.gen_range(pattern.len()..pattern.len() + (pattern.len() * scale));

        let pattern: Vec<char> = (0..picks)
            .map(|_| pattern[rng.gen_range(0..pattern.len())])
            .collect();

        let step = rng.gen_range(1..(PATTERNS.len() / 2).max(2));

        // Infinite wrapping pattern
        let size = pattern.len();
        let mut pattern = pattern.iter().cycle().step_by(step);

        // 50x50 random char sample
        self.cover = (0..50)
            .map(|_| {
                (0..50)
                    .map(|_| *(pattern.nth(rng.gen_range(0..size)).unwrap()))
                    .collect()
            })
            .collect();

        if rng.gen() {
            self.corners |= TLBR;
        }
        if rng.gen() {
            self.corners |= TRBL
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DeviceState {
    pub selection: usize,
    pub devices: Vec<Device>,
    pub end_action: Option<Action>,
}

impl DeviceState {
    pub fn set_devices(&mut self, devices: Vec<Device>) {
        self.devices = devices;
    }

    pub fn set_action(&mut self, action: Action) {
        self.end_action = Some(action);
    }

    pub fn select(&mut self, device: Option<Device>) {
        self.selection = 0;
        if let Some(device) = device {
            self.selection = self.devices.iter().position(|d| d == &device).unwrap_or(0);
        }
    }

    pub fn device(&self) -> Option<Device> {
        if self.devices.is_empty() {
            None
        } else {
            Some(self.devices[self.selection as usize].clone())
        }
    }

    pub fn reset(&mut self) {
        self.selection = 0;
        self.end_action = None;
    }

    pub fn next(&mut self) {
        if self.selection < self.devices.len().saturating_sub(1) {
            self.selection += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        }
    }
}

#[derive(Debug)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ModalWindow {
    #[default]
    DeviceSelect,
    Help,
    Exit,
    Error,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum MainWindow {
    #[default]
    Cover,
    Browse,
    Playlists,
    Queue,
    Library,
    Album,
    Artist,
    Show,
    AudioBook
}

/// State for what is currently focused
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub modal: ModalWindow,
    pub main: MainWindow,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            modal: ModalWindow::default(),
            main: MainWindow::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum WindowState {
    #[default]
    Main,
    Modal
}

#[derive(Debug, Clone)]
pub struct State {
    pub icons: IconsConfig,
    pub counter: u8,
    pub window: Window,
    pub window_state: WindowState,
    pub playback: PlaybackState,
    pub device_select: DeviceState,
}

impl State {
    pub async fn new() -> Self {
        Self {
            icons: IconsConfig::default(),
            counter: 0,
            window: Window::default(),
            window_state: WindowState::default(),
            device_select: DeviceState::default(),
            playback: PlaybackState::default(),
        }
    }

    pub fn back(&mut self) -> bool {
        match self.window_state {
            WindowState::Main => {
                match self.window.main {
                    MainWindow::Cover => return true,
                    _ => self.window.main = MainWindow::Cover,
                }
            },
            WindowState::Modal => self.window_state = WindowState::Main
        }
        false
    }

    pub fn show_modal(&mut self, modal: ModalWindow) {
        self.window.modal = modal;
        self.window_state = WindowState::Modal;
    }

    pub fn move_with(&mut self, movement: Move) {
        match self.window_state {
            WindowState::Modal => {
                match self.window.modal {
                    ModalWindow::DeviceSelect => {
                        match movement {
                            Move::Up => self.device_select.previous(),
                            Move::Down => self.device_select.next(),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
