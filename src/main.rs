use std::collections::HashMap;

use color_eyre::eyre::Result;
use crossterm::event::KeyCode;

use rataify::app::{Action, App};

#[tokio::main]
async fn main() -> Result<()> {
    App::new()
        .await?
        .run(HashMap::from([
            (KeyCode::Char(' ').into(), Action::Toggle),
            (KeyCode::Char('k').into(), Action::Increment),
            (KeyCode::Char('l').into(), Action::Increment),
            (KeyCode::Right.into(), Action::Increment),
            (KeyCode::Up.into(), Action::Increment),
            (KeyCode::Char('j').into(), Action::Decrement),
            (KeyCode::Char('h').into(), Action::Decrement),
            (KeyCode::Left.into(), Action::Decrement),
            (KeyCode::Down.into(), Action::Decrement),
            (KeyCode::Char('q').into(), Action::Quit),
            (KeyCode::Esc.into(), Action::Quit),
        ]))
        .await
}
