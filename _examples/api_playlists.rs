use std::env::args;

use tupy::{
    api::{
        Uri,
        auth::OAuth, flow::{Credentials, Pkce}, request::{PlaylistAction, PlaylistDetails}, response::Item, scopes, PublicApi, Spotify, UserApi
    },
    Pagination,
};

static ADO_SCREAMING_IN_MY_EAS: (&str, &str) = ("Ado screaming in my ear <3 ~ Dottore Hater", "2gYf8MKKfgCT14BJpZhS4r");
static ADO_ENGLISH_MIX: (&str, &str) = ("Ado English mix ~ Lulu", "1Vqadhez7eh6rey1vk7Yo2");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([
        scopes::USER_LIBRARY_READ,
        scopes::USER_LIBRARY_MODIFY,
        scopes::PLAYLIST_MODIFY_PUBLIC,
        scopes::PLAYLIST_MODIFY_PRIVATE,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;
    // Use a user defined playlist that they have access to. This is needed so that we can edit the
    // playlist.
    let playlist = spotify.api.playlist(args().nth(1).unwrap(), None).await?;

    let playlists = [ADO_SCREAMING_IN_MY_EAS, ADO_ENGLISH_MIX, (playlist.name.as_str(), playlist.id.as_str())];


    println!("[Playlist Items]");
    let mut items = spotify.api.playlist_items::<2, _, _>(&playlist.id, None)?;
    while let Some(page) = items.next().await? {
        for item in page.items {
            match item.item {
                Item::Track(track) => println!(" - {}", track.name),
                Item::Episode(episode) => println!(" - {}", episode.name),
            }
        }
    }
    println!();

    println!("Changed `{}` playlist to `Testing 1234`", playlist.name);
    spotify.api.update_playlist_details(&playlist.id, PlaylistDetails::new().name("Testing 1234")).await?;
    let _: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Press enter to continue")
        .allow_empty(true)
        .interact()?;
    spotify.api.update_playlist_details(&playlist.id, PlaylistDetails::new().name(&playlist.name)).await?;
    println!();

    if playlist.total_items < 2 {
        println!("[Warning] Reordering won't work with <= 1 items in the playlist");
    } else {
        println!("[Reorder Playlist '{}']", playlist.name);
        println!("  [Old]");
        let mut items = spotify.api.playlist_items::<2, _, _>(&playlist.id, None)?;
        while let Some(page) = items.next().await? {
            for item in page.items {
                match item.item {
                    Item::Track(track) => println!(" - {}", track.name),
                    Item::Episode(episode) => println!(" - {}", episode.name),
                }
            }
        }
        spotify.api.update_playlist_items(&playlist.id, PlaylistAction::Reorder {
            start: 1,
            length: 1,
            insert: 0,
        }).await?;
        println!("  [New]");
        let mut items = spotify.api.playlist_items::<2, _, _>(&playlist.id, None)?;
        while let Some(page) = items.next().await? {
            for item in page.items {
                match item.item {
                    Item::Track(track) => println!(" - {}", track.name),
                    Item::Episode(episode) => println!(" - {}", episode.name),
                }
            }
        }
        let _: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Press enter to continue")
            .allow_empty(true)
            .interact()?;
        spotify.api.update_playlist_items(&playlist.id, PlaylistAction::Reorder {
            start: 1,
            length: 1,
            insert: 0,
        }).await?;
        println!();
    }

    println!("[Add 'New Genesis ~ AmaLee' to {}]", playlist.name);
    let mut current_items = Vec::new();
    let mut items = spotify.api.playlist_items::<2, _, _>(&playlist.id, None)?;
    while let Some(page) = items.next().await? {
        for item in page.items {
            match item.item {
                Item::Track(track) => current_items.push(track.uri.clone()),
                Item::Episode(episode) => current_items.push(episode.uri.clone()),
            }
        }
    }
    let mut new_items = current_items.clone();
    new_items.push(Uri::track("2t2S7J0vj8WKaB9Pnmw6Re"));
    spotify.api.add_items(&playlist.id, [
        Uri::track("2t2S7J0vj8WKaB9Pnmw6Re")
    ], None).await?;
    let _: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Press enter to continue")
        .allow_empty(true)
        .interact()?;
    spotify.api.remove_items(&playlist.id, [
        Uri::track("2t2S7J0vj8WKaB9Pnmw6Re")
    ]).await?;
    println!();

    println!("[Featured Playlists]");
    let mut featured_playlists = spotify.api.featured_playlists::<2, _>(None)?;
    while let Some(page) = featured_playlists.next().await? {
        for playlist in page.items {
            println!(" - {}", playlist.name);
        }

        if featured_playlists.progress() >= 6 {
            if featured_playlists.progress() < featured_playlists.total() {
                println!(" - ...{} More...", featured_playlists.total() - featured_playlists.progress());
            }
            break;
        }
    }
    println!();

    println!("[Category 'party' Playlists]");
    let mut category_playlists = spotify.api.category_playlists::<2, _>("party")?;
    while let Some(page) = category_playlists.next().await? {
        for playlist in page.items {
            println!(" - {}", playlist.name);
        }

        if category_playlists.progress() >= 6 {
            if category_playlists.progress() < category_playlists.total() {
                println!(" - ...{} More...", category_playlists.total() - category_playlists.progress());
            }
            break;
        }
    }
    println!();

    println!("[Playlist Cover Image(s)]");
    for image in spotify.api.playlist_cover_image(&playlist.id).await? {
        println!(" - {}", image.url);
    }

    Ok(())
}
