use std::collections::HashMap;
use color_eyre::eyre::Result;
use rataify::{app::{Event, App}, key};

#[tokio::main]
async fn main() -> Result<()> {
    App::new()
        .await?
        .run(HashMap::from([
            // Menus
            (key!('d'), Event::OpenSelectDevice),
            (key!('g'), Event::OpenGoTo),
            (key!(','), Event::OpenAction),
            // TODO: Implement
            (key!('?'), Event::OpenHelp),
            // TODO: Implement
            (key!('/'), Event::OpenSearch),

            // Playback State
            (key!(' '), Event::Toggle),
            (key!('>' + SHIFT), Event::Next),
            (key!('<' + SHIFT), Event::Previous),
            (key!('r'), Event::ToggleRepeat),
            (key!('s'), Event::ToggleShuffle),
            (key!('+' + SHIFT), Event::VolumeUp),
            (key!('-'), Event::VolumeDown),

            // Navigation
            (key!(Enter), Event::Select),
            (key!(Right), Event::Right),
            (key!('l'), Event::Right),
            (key!(Left), Event::Left),
            (key!('h'), Event::Left),
            (key!(Up), Event::Up),
            (key!('k'), Event::Up),
            (key!(Down), Event::Down),
            (key!('j'), Event::Down),
            (key!(Tab), Event::Tab),
            (key!(BackTab + SHIFT), Event::Backtab),
            (key!('r' + CONTROL), Event::Refresh),
            (key!('R' + SHIFT + CONTROL), Event::Refresh),

            // Quit / Close
            (key!('q'), Event::Close),
            (key!('c' + CONTROL), Event::Quit),
            (key!('C' + SHIFT + CONTROL), Event::Quit),
        ]))
        .await
}
