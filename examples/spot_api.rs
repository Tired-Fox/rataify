use std::path::PathBuf;

use color_eyre::Result;
use dialoguer::{Confirm, Input, Password};
use serde_json::Value;
use rataify::spotify::{Credentials, OAuth, scopes};

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
    env_logger::init();


    // println!("{:?}", std::env::current_dir());
    // return;
    let dotenv = PathBuf::from(".env");
    if !dotenv.exists() {
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
        std::fs::write(dotenv, format!("RATAIFY_CLIENT_ID={client_id}\nRATAIFY_CLIENT_SECRET={client_secret}").as_bytes())?;
    }

    let creds = Credentials::from_env().unwrap();
    let mut oauth = OAuth::new(creds, scopes!("user-follow-read user-follow-modify"));

    oauth.refresh().await?;
    println!("{:?}", oauth.token());

    let client = reqwest::Client::new();
    let response = client.get("https://api.spotify.com/v1/me")
        .header("Authorization", oauth.token().unwrap().to_header())
        .send()
        .await?;

    let body = String::from_utf8(response.bytes().await?.to_vec())?;
    println!("{:#?}", serde_json::from_str::<Value>(&body)?);
    Ok(())
}