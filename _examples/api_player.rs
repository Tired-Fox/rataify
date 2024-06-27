use std::time::Duration;

use tupy::{api::{
    Uri,
    auth::OAuth, flow::{Credentials, Pkce}, request::{Play, Timestamp}, response::{Item, PlaybackAction, PlaybackActionScope, PlaybackItem, Repeat}, scopes, Spotify, UserApi
}, Pagination};

use crossterm::terminal;

fn play_pause(playing: bool) -> &'static str {
    if playing {
        "▷"
    } else {
        "⏸ "
    }
}

fn progress_bar(progress: i64, item: &PlaybackItem) -> String {
    let (cols, _rows) = terminal::size().unwrap();

    let duration = match item {
        PlaybackItem::Track(t) => t.duration.num_milliseconds(),
        PlaybackItem::Episode(e) => e.duration.num_milliseconds(),
        _ => 0,
    };

    // Num of milliseconds per column
    let scale = duration / cols as i64;
    format!("\x1b[32m{}\x1b[0m{}", (0..progress/scale).map(|_| "█").collect::<String>(), ((progress/scale)..(cols-1) as i64).map(|_| "█").collect::<String>())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([
        scopes::USER_READ_PLAYBACK_STATE,
        scopes::USER_MODIFY_PLAYBACK_STATE,
        scopes::USER_READ_CURRENTLY_PLAYING,
        scopes::USER_READ_RECENTLY_PLAYED,
    ])
    .unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;
    if let Some(playback) = spotify.api.playback_state(None).await? {
        println!(
            "{} {}",
            play_pause(playback.is_playing),
            match playback.item {
                PlaybackItem::Track(ref t) => t.name.as_ref(),
                PlaybackItem::Episode(ref e) => e.name.as_ref(),
                _ => "Ad",
            }
        );

        println!("{}", progress_bar(playback.progress.unwrap().num_milliseconds(), &playback.item));
        println!();
    }

    let playback = spotify.api.currently_playing(None).await?;
    if let Some(playback) = playback.as_ref() {
        println!(
            "{} {}",
            play_pause(playback.is_playing),
            match playback.item {
                PlaybackItem::Track(ref t) => t.name.as_ref(),
                PlaybackItem::Episode(ref e) => e.name.as_ref(),
                _ => "Ad",
            }
        );

        println!("{}", progress_bar(playback.progress.unwrap().num_milliseconds(), &playback.item));
        println!();
    }

    //spotify.api.play(Play::artist("6mEQK9m2krja6X1cfsAjfl"), None).await?; // Ado
    //spotify.api.play(Play::album("4muEF5biWb506ZojGMfHb7", None, 0), None).await?; // Ado ~ Kyougen
    //spotify.api.play(Play::playlist("1Vqadhez7eh6rey1vk7Yo2", None, 0), None).await?; // Ado English Mix
    //spotify.api.play(Play::queue([Uri::track("2ileXC69Z7xb95s3ljUBqb"), Uri::episode("5xewER743PrtdcT5qvXTeW")], 0), None).await?;

    if let Some(playback) = playback.as_ref() {
        if playback.is_playing {
            spotify.api.pause(None).await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
            spotify.api.play(Play::Resume, None).await?;
        } else {
            let devices = spotify.api.devices().await?;
            let index = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Select a device")
                .items(
                    &devices
                        .iter()
                        .map(|d| format!("[{:?}] {}", d.device_type, d.name))
                        .collect::<Vec<_>>(),
                )
                .default(0)
                .interact()?;
            let device = &devices[index];
            spotify.api.transfer_playback(&device.id, true).await?;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;

        spotify.api.next(None).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
        spotify.api.prev(None).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
        spotify.api.seek(0, None).await?;

        if let Some(actions) = playback.actions.get(&PlaybackActionScope::Disallows) {
            if !actions.contains_key(&PlaybackAction::TogglingShuffle) {
                spotify.api.shuffle(!playback.shuffle, None).await?;
            }
        }

        if let Some(actions) = playback.actions.get(&PlaybackActionScope::Disallows) {
            if !actions.contains_key(&PlaybackAction::TogglingRepeatContext) {
                spotify.api.repeat(Repeat::Context, None).await?;
            } else if !actions.contains_key(&PlaybackAction::TogglingRepeatTrack) {
                spotify.api.repeat(Repeat::Context, None).await?;
            } else {
                spotify.api.repeat(Repeat::Off, None).await?;
            }
        }

        if let Some(device) = &playback.device {
            if device.supports_volume {
                println!("Muting");
                spotify.api.volume(0, None).await?;
                tokio::time::sleep(Duration::from_secs(3)).await;
                println!("Restoring volume to {}%", device.volume_percent);
                spotify.api.volume(device.volume_percent, None).await?;
            } else {
                println!("Device does not support volume control");
            }
        }
    }

    let mut recently_played = spotify.api.recently_played::<2>(Timestamp::before_now())?;
    while let Some(page) = recently_played.next().await? {
        for history in page.items {
            println!(" - {} {}", history.played_at.format("%Y-%m-%d %H:%M:%S"), history.track.name);
        }

        if recently_played.progress() >= 6 {
            break;
        }
    }
    println!();

    println!("[QUEUE]");
    let queue = spotify.api.queue().await?;
    let cp = match &queue.currently_playing {
        Item::Track(t) => t.name.clone(),
        Item::Episode(e) => e.name.clone(),
    };
    println!(" - Currently playing: {}", cp);
    println!("{}", (0..20).map(|_| "-").collect::<String>());
    for item in queue.queue {
        match item {
            Item::Track(t) => println!(" - {}", t.name),
            Item::Episode(e) => println!(" - {}", e.name),
        }
    }
    println!();

    spotify.api.add_to_queue(Uri::track("2ileXC69Z7xb95s3ljUBqb"), None).await?;

    Ok(())
}
