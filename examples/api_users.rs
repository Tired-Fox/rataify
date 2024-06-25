use tupy::{
    api::{
        auth::OAuth,
        flow::{Credentials, Pkce},
        request::TimeRange,
        response::Track,
        scopes, PublicApi, Spotify, UserApi,
    },
    Pagination,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([
        scopes::USER_TOP_READ,
        scopes::PLAYLIST_MODIFY_PRIVATE,
        scopes::PLAYLIST_MODIFY_PUBLIC,
        scopes::USER_FOLLOW_READ,
        scopes::USER_FOLLOW_MODIFY,
    ])
    .unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let up = spotify.api.current_user_profile().await?;
    println!("[User]\n{:?}: {}\n", up.display_name, up.id);

    let mut top_items = spotify
        .api
        .user_top_items::<Track, 1>(TimeRange::Medium)?;
    println!("[Top Tracks]");
    while let Some(page) = top_items.next().await? {
        println!("Page {}", top_items.page() + 1);
        for item in page.items {
            println!(" - {}", item.name);
        }
        if top_items.page() >= 1 {
            break;
        }
    }
    println!();

    let up = spotify.api.user_profile(&up.id).await?;
    println!("[User]\n{:?}: {}\n", up.display_name, up.id);

    let playlist = "3cEYpjA9oz9GiPac4AsH4n";
    println!(
        "Add playlist ({playlist}): {}",
        spotify.api.follow_playlist(playlist, true).await.is_ok()
    );
    println!(
        "Now following playlist ({playlist}): {}",
        spotify.api.check_follow_playlist(playlist).await?
    );
    let _ = dialoguer::Confirm::new()
        .with_prompt("Continue to remove playlist?")
        .interact();
    println!(
        "Remove playlist ({playlist}): {}\n",
        spotify.api.unfollow_playlist(playlist).await.is_ok()
    );

    println!("[Followed Artists]");
    let mut followed_artists = spotify.api.followed_artists::<2>()?;
    while let Some(page) = followed_artists.next().await? {
        for artist in page.items {
            println!(" - {}", artist.name);
        }

        if followed_artists.item_count() >= 10 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    println!();

    let artists = [
        ("5Pwc4xIPtQLFEnJriah9YJ", "OneRepublic"),
        ("50iN4QFD9Z9S1KAzDLvAju", "Divide Music"),
    ];

    let names = artists.iter().map(|v| v.1).collect::<Vec<_>>();
    println!(
        "Follow artists {:?}: {}",
        names,
        spotify
            .api
            .follow_artists(artists.iter().map(|v| v.0))
            .await
            .is_ok()
    );
    println!("Now following artists:");
    for (artist, added) in names.iter().zip(
        spotify
            .api
            .check_follow_artists(artists.iter().map(|v| v.0))
            .await?
            .iter(),
    ) {
        println!(" - {}: {}", artist, added);
    }
    let _ = dialoguer::Confirm::new()
        .with_prompt("Continue to remove artists?")
        .interact();
    println!(
        "Unfollow artists {:?}: {}",
        names,
        spotify
            .api
            .unfollow_artists(artists.iter().map(|v| v.0))
            .await
            .is_ok()
    );

    Ok(())
}
