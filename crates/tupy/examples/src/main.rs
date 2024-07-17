use tupy::{api::{auth::OAuth, flow::{Credentials, Pkce}, request::Play, response::{Item, PlaybackActionScope, PlaybackAction}, scopes, PublicApi, Spotify, Uri, UserApi}, Pagination};
use tupy::api::request::{Query, SearchType};
use api_examples::util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let oauth = OAuth::from_env([
        scopes::USER_READ_PLAYBACK_STATE,
        scopes::USER_MODIFY_PLAYBACK_STATE,
        scopes::PLAYLIST_READ_PRIVATE,
        scopes::USER_READ_CURRENTLY_PLAYING,
        scopes::USER_LIBRARY_READ,
        scopes::USER_LIBRARY_MODIFY,
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

    //let queue = spotify.api.queue().await?;
    //for i in queue.queue.iter() {
    //    match i {
    //        Item::Track(track) => println!("- {}", track.name),
    //        Item::Episode(episode) => println!("- {}", episode.name),
    //    }
    //}

    //let pos = 3;
    //spotify.api.play(Play::queue(queue.queue.iter().skip(pos).map(|i| i.uri())), None).await?;
    
    //let mut saved_tracks = spotify.api.check_saved_tracks(queue.queue.iter().filter_map(|i| match i {
    //    Item::Track(track) => Some(track.id.clone()),
    //    _ => None
    //})).await?;
    //let mut saved_episodes = spotify.api.check_saved_episodes(queue.queue.iter().filter_map(|i| match i {
    //    Item::Episode(episode) => Some(episode.id.clone()),
    //    _ => None
    //})).await?;

    //let mut saved_tracks = saved_tracks.into_iter();
    //let mut saved_episodes = saved_episodes.into_iter();
    //for item in queue.queue.iter() {
    //    match item {
    //        Item::Track(track) => println!("- [{}] {}", saved_tracks.next().unwrap(), track.name),
    //        Item::Episode(episode) => println!("- [{}] {}", saved_episodes.next().unwrap(), episode.name),
    //    }
    //}

    //let mut episodes = spotify.api.show_episodes::<15, _, _>("5GcTIDkgnB9wP6CmUyOSqa", None)?;
    //if let Some(page) = episodes.next().await? {
    //    for episode in page.items {
    //        println!("- [{}] {}", episode.id, episode.name);
    //    }
    //}

    //let mut playlists = spotify.api.playlists::<40, _>(None)?;
    //while let Some(page) = playlists.next().await? {
    //    for playlist in page.items {
    //        println!("- [{}] {}", playlist.id, playlist.name);
    //    }
    //}

    let mut shows = spotify.api.saved_shows::<20>()?;
    while let Some(page) = shows.next().await? {
        for item in page.items {
            println!("[{}] {}", item.show.uri, item.show.name);
        }
    }
    
    //let mut search = spotify.api.search::<2, _>(&[Query::text("Release Radar")], &[SearchType::Playlist], None, false)?;
    //if let Some(playlists) = search.playlists() {
    //    if let Some(page) = playlists.next().await? {
    //        for playlist in page.items {
    //            if playlist.name.as_str() == "Release Radar" && playlist.owner.id.as_str() == "spotify" {
    //                println!("{:#?}", playlist);
    //            }
    //        }
    //    }
    //}
    //
    //let mut search = spotify.api.search::<2, _>(&[Query::text("Discover Weekly")], &[SearchType::Playlist], None, false)?;
    //if let Some(playlists) = search.playlists() {
    //    if let Some(page) = playlists.next().await? {
    //        for playlist in page.items {
    //            if playlist.name.as_str() == "Discover Weekly" && playlist.owner.id.as_str() == "spotify" {
    //                println!("{:#?}", playlist);
    //            }
    //        }
    //    }
    //}

    //let playback = spotify.api.playback_state(None).await?;
    //if let Some(playback) = playback {
    //    if playback.disallow(PlaybackAction::TogglingShuffle) {
    //        println!("Shuffle is disabled");
    //    }
    //
    //    if playback.disallow(PlaybackAction::TogglingRepeatContext) {
    //        println!("Repeat Context is disabled");
    //    }
    //
    //    if playback.disallow(PlaybackAction::TogglingRepeatTrack) {
    //        println!("Repeat Track is disabled");
    //    }
    //}

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
