use std::sync::{Arc, Mutex};

pub mod tui;
pub mod errors;
pub mod app;
pub mod ui;
pub mod spotify_util;
pub mod state;

pub type Shared<T> = Arc<T>;
pub type Locked<T> = Mutex<T>;

pub const PAGE_SIZE: usize = 30;

#[macro_export]
macro_rules! key {
    ($([$($state: ident),*])? $key:ident $(+ $mod:ident)* ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$key,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE $($(| crossterm::event::KeyEventState::$state)*)?,
            modifiers: crossterm::event::KeyModifiers::NONE $(| crossterm::event::KeyModifiers::$mod)*,
        }
    };
    ($([$($state: ident),*])? $key:literal $(+ $mod:ident)* ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($key),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE $($(| crossterm::event::KeyEventState::$state)*)?,
            modifiers: crossterm::event::KeyModifiers::NONE $(| crossterm::event::KeyModifiers::$mod)*
        }
    };
}
