use tupy::api::{auth::OAuth, flow::{Credentials, Pkce}, request::Play, scopes, Spotify, UserApi};
use api_examples::util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let oauth = OAuth::from_env([
        scopes::USER_READ_PLAYBACK_STATE,
        scopes::USER_MODIFY_PLAYBACK_STATE,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(
        Credentials::from_env().unwrap(), 
        oauth,
        "tupy"
    )?;

    util::refresh_token(&spotify.api).await?;

    let playback = spotify.api.playback_state(None).await?;
    match playback {
        Some(playback) => {
            match playback.is_playing {
                true => spotify.api.pause(None).await?,
                false => spotify.api.play(Play::Resume, None).await?,
            }
        },
        None => {
            let devices = spotify.api.devices().await?;
            let device = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Select a device")
                .items(&devices.iter().map(|d| {
                    format!("{} ({:?})", d.name, d.device_type)
                }).collect::<Vec<_>>())
                .default(0)
                .interact()?;
            let device = devices.get(device).unwrap();
            spotify.api.play(Play::Resume, device.id.clone()).await?;
        }
    }

    Ok(())
}
