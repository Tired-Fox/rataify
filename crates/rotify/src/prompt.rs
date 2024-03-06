use std::default::Default;

use dialoguer::{Confirm, Input, Password};

use crate::CONFIG_PATH;

static BEFORE_ID: [&str; 6] = [
    "Click 'Create app' button",
    "Name the app whatever you want",
    "Add a \x1b[33mRedirect URI\x1b[39m of \x1b[36mhttp://localhost:8888/Rotify/auth\x1b[39m",
    "Enable the following API/SDKs: 'Web API' and 'Web Playback SDK'",
    "Agree to Spotify's \x1b]8;;https://developer.spotify.com/terms\x1b\\\x1b[36mDeveloper Terms of Service\x1b[39m\x1b]8;;\x1b\\ and \x1b]8;;https://developer.spotify.com/documentation/design\x1b\\\x1b[36mDesign Guidelines\x1b[39m\x1b]8;;\x1b\\",
    "Should redirect to app landing page, click on the settings button on the top right",
];

static BEFORE_SECRET: [&str; 1] = [
    "Click 'View client secret'. \x1b[33;1mWARNING\x1b[39m do not share this value with anyone\x1b[22m"
];

static BANNER: [&str; 6] = [
    "██████╗  ██████╗ ████████╗██╗███████╗██╗   ██╗",
    "██╔══██╗██╔═══██╗╚══██╔══╝██║██╔════╝╚██╗ ██╔╝",
    "██████╔╝██║   ██║   ██║   ██║█████╗   ╚████╔╝",
    "██╔══██╗██║   ██║   ██║   ██║██╔══╝    ╚██╔╝",
    "██║  ██║╚██████╔╝   ██║   ██║██║        ██║",
    "╚═╝  ╚═╝ ╚═════╝    ╚═╝   ╚═╝╚═╝        ╚═╝",
];

pub(crate) fn prompt_creds_if_missing() -> color_eyre::Result<()> {
    if !CONFIG_PATH.join(".env").exists() {
        if let Some((terminal_size::Width(width), terminal_size::Height(height))) = terminal_size::terminal_size() {
            println!("\n");
            let prefix = " ".repeat((width-46) as usize /2);
            for line in BANNER.iter() {
                println!("{}{}", prefix, line);
            }
            println!("\n");
        }

        if Confirm::new()
            .with_prompt("Setup requires interating with the spotify dashboard. Open spotify dashboard \x1b[36mhttps://developer.spotify.com/dashboard\x1b[39m")
            .interact()
            .unwrap() {
            open::that("https://developer.spotify.com/dashboard")?;
        }
        for (i, step) in BEFORE_ID.iter().enumerate() {
            println!("{}. {}", i + 1, step);
        }
        let client_id: String = Input::new()
            .with_prompt(format!("{}. Copy and paste Spotify client ID here", BEFORE_ID.len() + 1))
            .interact_text()
            .unwrap();
        for (i, step) in BEFORE_SECRET.iter().enumerate() {
            println!("{}. {}", i + BEFORE_ID.len() + 2, step);
        }
        let client_secret: String = Password::new()
            .with_prompt(format!("{}. Copy and paste Spotify client secret here (hidden)", BEFORE_ID.len() + BEFORE_SECRET.len() + 2))
            .interact()
            .unwrap();

        std::fs::create_dir_all(CONFIG_PATH.as_path())?;
        std::fs::write(CONFIG_PATH.join(".env"), format!("RATAIFY_CLIENT_ID={client_id}\nRATAIFY_CLIENT_SECRET={client_secret}").as_bytes())?;
    }
    Ok(())
}