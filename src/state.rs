use std::fmt::{Debug, Display};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use std::sync::Mutex;
use chrono::Duration;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui_image::picker::Picker;

use crate::spotify::api::{Episode, Item, Playback, Track};
use crate::spotify::Spotify;

lazy_static::lazy_static! {
    pub static ref PICKER: Mutex<Picker> = {
        #[cfg(windows)]
        return Mutex::new(Picker::new((8, 16)));

        #[cfg(not(windows))]
        return Mutex::new({
            let mut picker = Picker::from_termios().unwrap();
            picker.guess_protocol().unwrap();
            picker
        });
    };

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

#[derive(Debug)]
pub struct PlaybackState {
    pub cover: Vec<String>,
    pub current: Option<Playback>,

    corners: Flag, // 1 = tl + br, 2 = tr + bl
}

impl Default for PlaybackState {
    fn default() -> Self {
        let mut state = Self {
            cover: Vec::new(),
            current: None,
            corners: Flag::default(),
        };
        state.generate_cover();
        state
    }
}

impl PlaybackState {
    pub fn now_playing(&mut self, playback: Option<Playback>)
    {
        self.corners = Flag::default();
        self.current = playback;

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
        let progress = self.progress().num_milliseconds();
        let duration = self.duration().num_milliseconds();

        if progress == 0 || duration == 0 {
            return 0.0;
        } else {
            progress as f64 / duration as f64
        }
    }

    pub fn progress(&self) -> Duration {
        match &self.current {
            Some(Playback { progress, .. }) => progress.unwrap_or(Duration::zero()).clone(),
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

    // TODO: Randomize border based on title and artist.
    //  Corners based on artist, sides based on title
    pub fn cover(&self, height: usize) -> String {
        let height = height - 2;
        let width: usize = (height as f32 * 2.5) as usize;
        let mut output = format!(
            "┌─{}─┐\n",
            // if self.corners & TLBR == TLBR { "┌─" } else { "  " },
            " ".repeat(width - 2),
            // if self.corners & TRBL == TRBL { "─┐" } else { "  " },
        );
        output.push_str(
            format!(
                "{}",
                self.cover
                    .iter()
                    .skip((50 - height) / 2)
                    .take(height)
                    .map(|r| format!(
                        " {} ",
                        r.chars()
                            .skip((50 - width) / 2)
                            .take(width)
                            .collect::<String>(),
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
                .as_str(),
        );
        output.push_str(format!(
            "\n└─{}─┘",
            //'┌', '┐', '└', '┘'
            // if self.corners & TRBL == TRBL { "└─" } else { "  " },
            " ".repeat(width - 2),
            // if self.corners & TLBR == TLBR { "─┘" } else { "  " },
        ).as_str());
        output
    }

    fn generate_cover(&mut self) {
        if self.current.is_none() {
            self.cover = (0..50).map(|_| " ".repeat(50)).collect();
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
                    .map(|_| pattern.nth(rng.gen_range(0..size)).unwrap())
                    .collect::<String>()
            })
            .collect::<Vec<String>>();

        if rng.gen() {
            self.corners |= TLBR;
        }
        if rng.gen() {
            self.corners |= TRBL
        }
    }
}

pub struct State {
    pub counter: u8,
    pub playback: PlaybackState,
    pub spotify: Spotify,
}

impl State {
    pub async fn new() -> Self {
        Self {
            counter: 0,
            playback: PlaybackState::default(),
            spotify: Spotify::new().await.unwrap(),
        }
    }

    pub(crate) async fn next(&mut self) {
        self.playback.now_playing(self.spotify.playback().await.ok());
    }

    pub(crate) async fn previous(&mut self) {
        self.playback.now_playing(self.spotify.playback().await.ok());
    }
}
