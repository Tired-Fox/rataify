
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use rataify::{key, Key};

fn main() {
    println!("{}", Key(KeyEvent {
        code: KeyCode::Char('A'),
        modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }) == key!('A' + SHIFT))
}
