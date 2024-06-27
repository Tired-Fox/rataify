use tupy::{
    api::{
        auth::OAuth, flow::{Credentials, Pkce}, request::{RecommendationSeed, SeedId}, scopes, PublicApi, Spotify, UserApi
    },
    Pagination,
};

static THE_MOUNTAIN_SONG: (&str, &str) = ("The Mountain Song ~ Tophouse", "2ileXC69Z7xb95s3ljUBqb");
static PROJECT: (&str, &str) = ("Project ~ Chase McDaniel", "6TpqWzySgiHLoXvQvyGtqH");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([scopes::USER_LIBRARY_READ, scopes::USER_LIBRARY_MODIFY]).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;
    let tracks = [THE_MOUNTAIN_SONG, PROJECT];

    let track = spotify.api.track(THE_MOUNTAIN_SONG.1, None).await?;
    println!("{}", track.name);
    println!();

    println!("[Tracks]");
    for track in spotify.api.tracks(tracks.iter().map(|e| e.1), None).await? {
        println!(
            " - {} ~ {}",
            track.name,
            track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();

    let mut saved_tracks = spotify.api.saved_tracks::<2, _>(None)?;
    println!("[Saved Tracks]");
    while let Some(page) = saved_tracks.next().await? {
        for saved_track in page.items {
            println!(" - {}", saved_track.track.name);
        }

        if saved_tracks.progress() >= 6 {
            break;
        }
    }
    println!();

    let names = tracks.iter().map(|e| e.0).collect::<Vec<&str>>();
    println!("Saving tracks {names:?}");
    spotify.api.save_shows(tracks.iter().map(|e| e.1)).await?;
    println!();

    println!("[Tracks Saved]");
    for (track, added) in names.iter().zip(
        spotify
            .api
            .check_saved_shows(tracks.iter().map(|e| e.1))
            .await?,
    ) {
        println!(" - {}: {}", track, added);
    }
    println!();

    _ = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Remove shows {names:?}?"))
        .interact()?;

    println!("Remove shows {names:?}");
    spotify
        .api
        .remove_saved_shows(tracks.iter().map(|e| e.1))
        .await?;
    println!();

    println!("[Audio Features (Multi)]");
    for features in spotify
        .api
        .track_audio_features(tracks.iter().map(|e| e.1))
        .await?
    {
        println!(
            " - {:0>2}:{:0>2} ~ ::{}::",
            features.duration.num_minutes() % 60,
            features.duration.num_seconds() % 60,
            features.energy
        );
    }
    println!();

    println!("[Audio Features (Single)]");
    let features = spotify.api.track_audio_feature(THE_MOUNTAIN_SONG.1).await?;
    println!(
        " - {:0>2}:{:0>2} ~ ::{}::",
        features.duration.num_minutes() % 60,
        features.duration.num_seconds() % 60,
        features.energy
    );
    println!();

    println!("[Audio Analysis]");
    let analysis = spotify
        .api
        .track_audio_analysis(THE_MOUNTAIN_SONG.1)
        .await?;
    println!(
        " - {} ({}) ~ {:0>2}:{:0>2}",
        analysis.meta.platform,
        analysis.meta.status_code,
        analysis.meta.analysis_time.num_minutes() % 60,
        analysis.meta.analysis_time.num_seconds() % 60
    );
    println!();

    println!("[Recommendations]");
    for track in  spotify.api.recommendations::<10, _>(None, RecommendationSeed {
        seed_ids: vec![SeedId::track(THE_MOUNTAIN_SONG.1), SeedId::track(PROJECT.1)],
        ..Default::default()
    }).await?.tracks {
        println!(" - {} ~ {}", track.name, track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", "));
    }

    Ok(())
}
