mod tui;
mod app;
mod error;
pub mod api;
pub mod action;
pub mod state;
pub mod event;

pub use tui::Tui;
pub use app::App;
pub use error::{Error, ErrorKind};

#[macro_export]
macro_rules! keyevent {
    ($({ $($modifier: ident)* })? $key: literal) => {
       crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char($key),
            crossterm::event::KeyModifiers::empty() $($(| crossterm::event::KeyModifiers::$modifier)*)?
       ) 
    };
    ($({ $($modifier: ident)* })? $key: ident) => {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::$key,
            crossterm::event::KeyModifiers::empty() $($(| crossterm::event::KeyModifiers::$modifier)*)?
        ) 
    };
}
