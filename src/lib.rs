mod tui;
mod app;
mod error;
pub mod config;
pub mod api;
pub mod action;
pub mod state;
pub mod event;

use crossterm::event::KeyCode;
use rspotify::model::{CursorBasedPage, Page};
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
    ($({ $($modifier: ident)* })? F ($F: literal)) => {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F($F),
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

pub trait ConvertPage<T> {
    fn convert_page(self) -> Page<T>;
}

impl<A, B: From<A>> ConvertPage<B> for Page<A> {
    fn convert_page(self) -> Page<B> {
        Page {
            href: self.href,
            limit: self.limit,
            next: self.next,
            offset: self.offset,
            previous: self.previous,
            total: self.total,
            items: self.items.into_iter().map(B::from).collect(),
        }
    }
}

impl<A, B: From<A>> ConvertPage<B> for CursorBasedPage<A> {
    fn convert_page(self) -> Page<B> {
        Page {
            href: self.href,
            limit: self.limit,
            next: self.next,
            offset: 0,
            previous: None,
            total: self.total.unwrap_or_default(),
            items: self.items.into_iter().map(B::from).collect(),
        }
    }
}
