use color_eyre::Result;

use rataify::action::Public;
use rataify::app::App;
use rataify::ui::player_ui;
use rataify::{config::Config, keymaps};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config: Config = Config::load_with_fallback(["config.yml", "config.yaml"])?
        .reserved_keys(keymaps! {
            "ctrl+c" => Public::Exit,
        })
        .default_keys(keymaps! {
            "q" => Public::Back,
            "left" => Public::Left,
            "right" => Public::Right,
            "up" => Public::Up,
            "down" => Public::Down,
            "enter" => Public::Select,
        })
        .compile();

    App::new().await?.with_ui(player_ui).run(config).await
}
