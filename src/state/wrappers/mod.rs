use std::fmt::Debug;

use crossterm::event::KeyEvent;
use tupy::api::{response::{Episode, Track}, Uri};

use crate::{key, state::actions::{Action, action_label, IntoActions}};

pub trait GetUri {
    fn get_uri(&self) -> Uri;
}

impl GetUri for Track {
    fn get_uri(&self) -> Uri {
        self.uri.clone()
    }
}
impl GetUri for Episode {
    fn get_uri(&self) -> Uri {
        self.uri.clone()
    }
}

pub struct Saved<T> {
    pub saved: bool,
    inner: T
}

impl<T: PartialEq> PartialEq for Saved<T> {
    fn eq(&self, other: &Self) -> bool {
        self.saved == other.saved && self.inner.eq(other.as_ref())
    }
}
impl<T: PartialEq> PartialEq<T> for Saved<T> {
    fn eq(&self, other: &T) -> bool {
        self.inner.eq(other)
    }
}

impl<T: Debug> Debug for Saved<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Saved")
            .field("saved", &self.saved)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Clone> Clone for Saved<T> {
    fn clone(&self) -> Self {
        Self {
            saved: self.saved,
            inner: self.inner.clone()
        }
    }
}

impl<T: IntoActions + GetUri> Saved<T> {
    pub fn into_ui_actions<F>(&self, context: bool, callback: F) -> Vec<(KeyEvent, Action, &'static str)>
    where
        F: Fn(bool) -> color_eyre::Result<()> + Send + Sync + 'static
    {
        let mut actions = vec![
            if self.saved {
                (key!('r'), Action::remove(self.inner.get_uri(), callback), action_label::REMOVE)
            } else {
                (key!('f'), Action::save(self.inner.get_uri(), callback), action_label::SAVE)
            }
        ];
        actions.extend(self.inner.into_ui_actions(context));
        actions
    }
}

impl<T> std::convert::AsRef<T> for Saved<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> std::convert::AsMut<T> for Saved<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Saved<T> {
    pub fn new(saved: bool, inner: T) -> Self {
        Self { saved, inner } 
    }

    pub fn unwrap(self) -> T {
        self.inner
    }
}
