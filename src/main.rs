use rataify::{action::Action, keyevent, App, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut app = App::new([
        (keyevent!('q'), Action::Close),
        (keyevent!(Tab), Action::Tab),
        (keyevent!(BackTab), Action::BackTab),

        (keyevent!('h'), Action::Left),
        (keyevent!('j'), Action::Down),
        (keyevent!('k'), Action::Up),
        (keyevent!('l'), Action::Right),

        (keyevent!(Up), Action::Up),
        (keyevent!(Down), Action::Down),
        (keyevent!(Left), Action::Left),
        (keyevent!(Right), Action::Right),

        (keyevent!(Enter), Action::Select),

        (keyevent!(' '), Action::Toggle),
        (keyevent!('n'), Action::Next),
        (keyevent!('p'), Action::Previous),
        (keyevent!('s'), Action::Shuffle),
        (keyevent!('r'), Action::Repeat),
    ]).await?;

    app.run().await
}
