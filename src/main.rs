use color_eyre::Result;

use rataify::{config::Config, keymaps};
use rataify::action::Public;
use rataify::app::App;
use rataify::ui::{mock_player};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config: Config = Config::load_with_fallback(["config.yml", "config.yaml"])?
        .reserved_keys(keymaps! {
            "ctrl+c" => Public::Exit,
            "ctrl+shift+z" => Public::Exit,
            "q" => Public::Close,
            "left" => Public::Left,
            "right" => Public::Right,
            "up" => Public::Up,
            "down" => Public::Down,
            "enter" => Public::Select,
        })
        .compile();

    let result = App::new().await?
        .with_ui(mock_player)
        .run(config)
        .await;

    result
}
