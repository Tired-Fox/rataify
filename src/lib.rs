mod tui;
mod key;
mod uri;
mod app;
mod error;
pub mod config;
pub mod api;
pub mod action;
pub mod state;
pub mod event;

use rspotify::model::{CursorBasedPage, Offset, Page};
pub use tui::Tui;
pub use app::App;
pub use error::{Error, ErrorKind};
use uri::Uri;

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

pub trait IntoSpotifyParam<T, S = ()> {
    fn into_spotify_param(self) -> T;
}

struct OptionalSpotifyParam;
impl<F, I: IntoSpotifyParam<F>> IntoSpotifyParam<Option<F>, OptionalSpotifyParam> for Option<I> {
    fn into_spotify_param(self) -> Option<F> {
        self.map(|s| s.into_spotify_param())
    }
}

struct NoSpotifyParam;
impl<F> IntoSpotifyParam<Option<F>, NoSpotifyParam> for Option<()> {
    fn into_spotify_param(self) -> Option<F> {
        None
    }
}

impl<F, I: IntoSpotifyParam<F>> IntoSpotifyParam<Option<F>> for I {
    fn into_spotify_param(self) -> Option<F> {
        Some(self.into_spotify_param())
    }
}

impl IntoSpotifyParam<Offset> for usize {
    fn into_spotify_param(self) -> Offset {
        Offset::Position(chrono::Duration::milliseconds(self as i64))
    }
}

impl IntoSpotifyParam<Offset> for &str {
    fn into_spotify_param(self) -> Offset {
        Offset::Uri(self.to_string())
    }
}

impl IntoSpotifyParam<Offset> for String {
    fn into_spotify_param(self) -> Offset {
        Offset::Uri(self)
    }
}

impl IntoSpotifyParam<Offset> for Uri {
    fn into_spotify_param(self) -> Offset {
        Offset::Uri(self.to_string())
    }
}

impl IntoSpotifyParam<chrono::Duration> for usize {
    fn into_spotify_param(self) -> chrono::Duration {
        chrono::Duration::milliseconds(self as i64)
    }
}
