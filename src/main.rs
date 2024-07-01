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
            (key!(' '), Action::Toggle),
            (key!(Enter), Action::Select),
            (key!('d'), Action::SelectDevice),

            (key!('>' + SHIFT), Action::Next),
            (key!('<' + SHIFT), Action::Previous),

            (key!(Tab), Action::Tab),
            (key!(Tab + SHIFT), Action::PrevTab),

            (key!(Right), Action::Right),
            (key!('l'), Action::Left),
            (key!(Left), Action::Left),
            (key!('h'), Action::Left),
            (key!(Up), Action::Up),
            (key!('k'), Action::Up),
            (key!(Down), Action::Down),
            (key!('j'), Action::Down),
            (key!('q'), Action::Close),
        ]))
        .await
}
