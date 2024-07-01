use tupy::{api::{auth::OAuth, flow::{Credentials, Pkce}, request::Play, scopes, PublicApi, Spotify, Uri, UserApi}, Pagination};
use api_examples::util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let oauth = OAuth::from_env([
        scopes::USER_READ_PLAYBACK_STATE,
        scopes::USER_MODIFY_PLAYBACK_STATE,
        scopes::PLAYLIST_READ_PRIVATE,
        scopes::USER_READ_CURRENTLY_PLAYING,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(
        Credentials::from_env().unwrap(), 
        oauth,
        "tupy"
    )?;
    

    util::refresh_token(&spotify.api).await?;
    // 7AXnDxOcbYCymLv2krA3Hx
    //spotify.api.play(Play::playlist("7AXnDxOcbYCymLv2krA3Hx", Some(2), 0), None).await?;

    // 5GcTIDkgnB9wP6CmUyOSqa
    //spotify.api.play(Play::show("5GcTIDkgnB9wP6CmUyOSqa", None, 0), None).await?;

    // 2PVAFANvXwdtNvxS5NoNEt
    //spotify.api.play(Play::queue([Uri::episode("2PVAFANvXwdtNvxS5NoNEt")], None, 0), None).await?;

    println!("{:#?}", spotify.api.queue().await?);

    //let mut episodes = spotify.api.show_episodes::<15, _, _>("5GcTIDkgnB9wP6CmUyOSqa", None)?;
    //if let Some(page) = episodes.next().await? {
    //    for episode in page.items {
    //        println!("- [{}] {}", episode.id, episode.name);
    //    }
    //}

    //let mut playlists = spotify.api.playlists::<20, _>(None)?;
    //while let Some(page) = playlists.next().await? {
    //    for playlist in page.items {
    //        println!("- [{}] {}", playlist.id, playlist.name);
    //    }
    //}

    //
    //let playback = spotify.api.playback_state(None).await?;
    //match playback {
    //    Some(playback) => {
    //        match playback.is_playing {
    //            true => spotify.api.pause(None).await?,
    //            false => spotify.api.play(Play::Resume, None).await?,
    //        }
    //    },
    //    None => {
    //        let devices = spotify.api.devices().await?;
    //        let device = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
    //            .with_prompt("Select a device")
    //            .items(&devices.iter().map(|d| {
    //                format!("{} ({:?})", d.name, d.device_type)
    //            }).collect::<Vec<_>>())
    //            .default(0)
    //            .interact()?;
    //        let device = devices.get(device).unwrap();
    //        spotify.api.play(Play::Resume, device.id.clone()).await?;
    //    }
    //}
    //
    Ok(())
}
