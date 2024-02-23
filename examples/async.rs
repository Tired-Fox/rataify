use color_eyre::Result;

use rataify::{config::Config, keymaps};
use rataify::action::Public;
use rataify::app::App;
use rataify::ui::counter;

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = Config::load_with_fallback(["config.yml", "config.yaml"])?
        .reserved_keys(keymaps! {
            "ctrl+c" => Public::Exit,
            "ctrl+shift+z" => Public::Exit,
            "q" => Public::Close,
        })
        .compile();

    App::new()?
        .with_ui(counter)
        .run(config).await?;

    Ok(())
}
