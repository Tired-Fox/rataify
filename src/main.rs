use rataify::{action::Action, App, Error};

macro_rules! keyevent {
    ($({ $($modifier: ident)* })? $key: literal) => {
       crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char($key),
            crossterm::event::KeyModifiers::empty() $($(| crossterm::event::KeyModifiers::$modifier)*)?
       ) 
    };
    ($({ $($modifier: ident)* } $key: ident)?) => {
       crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::$key,
            crossterm::event::KeyModifiers::empty() $($(| crossterm::event::KeyModifiers::$modifier)*)?
       ) 
    };
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut app = App::new([
        (keyevent!('q'), Action::Quit)
    ]).await?;

    app.run().await
}
