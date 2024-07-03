use std::collections::HashMap;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use rataify::app::{Action, App};

macro_rules! key {
    ($([$($state: ident),*])? $key:ident $(+ $mod:ident)* ) => {
        KeyEvent {
            code: KeyCode::$key,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE $($(| KeyEventState::$state)*)?,
            modifiers: KeyModifiers::NONE $(| KeyModifiers::$mod)*,
        }
    };
    ($([$($state: ident),*])? $key:literal $(+ $mod:ident)* ) => {
        KeyEvent {
            code: KeyCode::Char($key),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE $($(| KeyEventState::$state)*)?,
            modifiers: KeyModifiers::NONE $(| KeyModifiers::$mod)*
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new()
        .await?
        .run(HashMap::from([
            // Menus
            (key!('d'), Action::OpenSelectDevice),
            (key!('g'), Action::OpenGoTo),
            // TODO: Implement
            (key!(','), Action::OpenAction),
            // TODO: Implement
            (key!('?'), Action::OpenHelp),
            // TODO: Implement
            (key!('/'), Action::OpenSearch),

            // Playback State
            (key!(' '), Action::Toggle),
            (key!('>' + SHIFT), Action::Next),
            (key!('<' + SHIFT), Action::Previous),
            (key!('r'), Action::ToggleRepeat),
            (key!('s'), Action::ToggleShuffle),
            (key!('+' + SHIFT), Action::VolumeUp),
            (key!('-'), Action::VolumeDown),

            // Navigation
            (key!(Enter), Action::Select),
            (key!(Right), Action::Right),
            (key!('l'), Action::Left),
            (key!(Left), Action::Left),
            (key!('h'), Action::Left),
            (key!(Up), Action::Up),
            (key!('k'), Action::Up),
            (key!(Down), Action::Down),
            (key!('j'), Action::Down),

            // Quit / Close
            (key!('q'), Action::Close),
            (key!('c' + CONTROL), Action::Quit),
            (key!('C' + SHIFT + CONTROL), Action::Quit),
        ]))
        .await
}
