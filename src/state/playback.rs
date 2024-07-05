use std::collections::HashMap;

use tupy::{api::response::{self, Device, PlaybackAction, PlaybackActionScope, PlaybackItem, Repeat}, DateTime, Duration, Local};

#[derive(Debug, Clone, PartialEq)]
pub struct Playback {
    pub saved: bool,
    pub device: Option<Device>,
    pub repeat: Repeat,
    pub shuffle: bool,
    pub progress: Option<Duration>,
    pub is_playing: bool,
    pub item: PlaybackItem,
    pub actions: HashMap<PlaybackActionScope, HashMap<PlaybackAction, bool>>,
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

    /// True if the action is disallowed
    pub fn disallow(&self, actions: PlaybackAction) -> bool {
        if self.actions.contains_key(&PlaybackActionScope::Disallows) {
            return self.actions.get(&PlaybackActionScope::Disallows).unwrap().contains_key(&actions);
        }
        false
    }

    /// True if all actions are disallowed
    pub fn disallows<I: IntoIterator<Item=PlaybackAction>>(&self, actions: I) -> bool {
        if self.actions.contains_key(&PlaybackActionScope::Disallows) {
            let disallows = self.actions.get(&PlaybackActionScope::Disallows).unwrap();
            return actions.into_iter().all(|action| disallows.contains_key(&action));
        }
        false
    }
}

impl From<response::Playback> for Playback {
    fn from(pb: response::Playback) -> Self {
        Self {
            saved: false,
            device: pb.device,
            repeat: pb.repeat,
            shuffle: pb.shuffle,
            progress: pb.progress,
            is_playing: pb.is_playing,
            item: pb.item,
            actions: pb.actions,
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PlaybackState {
    pub last_playback_poll: DateTime<Local>,
    pub playback: Option<Playback>,
}

impl PlaybackState {
    pub fn new(playback: Option<Playback>) -> Self {
        Self {
            last_playback_poll: Local::now(),
            playback,
        }
    }

    pub fn set_playback(&mut self, playback: Option<Playback>) -> bool {
        self.last_playback_poll = Local::now();
        let diff = self.playback != playback;
        let saved = self.playback.as_ref().map(|pb| pb.saved).unwrap_or(false);

        self.playback = playback;
        self.set_saved(saved);
        diff
    }

    pub fn set_saved(&mut self, saved: bool) {
        if let Some(playback) = self.playback.as_mut() {
            playback.saved = saved;
        }
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        self.playback.is_some()
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.playback.is_none()
    }
}

