use color_eyre::Result;
use dialoguer::{Confirm, Input, Password, Select};
use dialoguer::theme::ColorfulTheme;

use rataify::CONFIG_PATH;
use rataify::spotify::body::TransferPlayback;
use rataify::spotify::Spotify;

static BEFORE_ID: [&str; 6] = [
    "Click 'Create app' button",
    "Name the app whatever you want",
    "Add a \x1b[33mRedirect URI\x1b[39m of \x1b[36mhttp://localhost:8888/callback\x1b[39m",
    "Enable the following API/SDKs: 'Web API' and 'Web Playback SDK'",
    "Agree to Spotify's \x1b]8;;https://developer.spotify.com/terms\x1b\\\x1b[36mDeveloper Terms of Service\x1b[39m\x1b]8;;\x1b\\ and \x1b]8;;https://developer.spotify.com/documentation/design\x1b\\\x1b[36mDesign Guidelines\x1b[39m\x1b]8;;\x1b\\",
    "Should redirect to app landing page, click on the settings button on the top right",
];

static BEFORE_SECRET: [&str; 1] = [
    "Click 'View client secret'. \x1b[33;1mWARNING\x1b[39m do not share this value with anyone\x1b[22m"
];


#[tokio::main]
async fn main() -> Result<()> {
    // println!("{:?}", std::env::current_dir());
    // return;
    if !CONFIG_PATH.join(".env").exists() {
        println!("Missing id and secret for spotify app. Setup cli activating...");
        if Confirm::new()
            .with_prompt("Auto open spotify dashboard \x1b[36mhttps://developer.spotify.com/dashboard\x1b[39m")
            .interact()
            .unwrap() {
            open::that("https://developer.spotify.com/dashboard")?;
        }
        println!("Spotify Developer Dashboard: \x1b]8;;https://developer.spotify.com/dashboard\x1b\\https://developer.spotify.com/dashboard\x1b]8;;\x1b\\");
        println!("If you don't already have an application follow these steps. Otherwise select the app and start at ({}):", BEFORE_ID.len());
        for (i, step) in BEFORE_ID.iter().enumerate() {
            println!("{}. {}", i+1, step);
        }
        let client_id: String = Input::new()
            .with_prompt(format!("{}. Copy and paste Spotify client ID here", BEFORE_ID.len()+1))
            .interact_text()
            .unwrap();
        for (i, step) in BEFORE_SECRET.iter().enumerate() {
            println!("{}. {}", i+BEFORE_ID.len()+2, step);
        }
        let client_secret: String = Password::new()
            .with_prompt(format!("{}. Copy and paste Spotify client secret here (hidden)", BEFORE_ID.len() + BEFORE_SECRET.len() + 2))
            .interact()
            .unwrap();
        std::fs::write(CONFIG_PATH.join(".env"), format!("RATAIFY_CLIENT_ID={client_id}\nRATAIFY_CLIENT_SECRET={client_secret}").as_bytes())?;
    }

    let mut spotify = Spotify::new().await?;

    let devices = spotify.devices().await?.clone();
    let names = devices.iter().map(|d| &d.name).collect::<Vec<&String>>();

    let device = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a Device")
        .items(names.as_slice())
        .interact()
        .unwrap();

    spotify.device.select(device);
    spotify.transfer_playback(&TransferPlayback {
        device_ids: [devices.get(device).unwrap().id.clone()],
        play: None,
    }).await?;
    // spotify.play().await?;
    // let response = rataify::spotify::api::SpotifyRequest::new()
    //     .url("/me/player")
    //     .send(&mut spotify.oauth)
    //     .await?;
    // println!("{:#?}", response.json::<Playback>().await?);
    // println!("{:?}", spotify.next().await);
    Ok(())
}