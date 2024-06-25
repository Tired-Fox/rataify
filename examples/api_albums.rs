use tupy::{
    api::{
        auth::OAuth, flow::{Pkce, Credentials}, request::Market, scopes, PublicApi, Spotify, UserApi
    },
    Pagination,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let oauth = OAuth::from_env([
        scopes::USER_LIBRARY_READ,
        scopes::USER_LIBRARY_MODIFY,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let mut top_items = spotify.api.new_releases::<2>()?;
    while let Some(page) = top_items.next().await? {
        println!("Page {}", top_items.page() + 1);
        for item in page.items {
            println!(" - {}", item.name);
        }
        if top_items.item_count() >= 4 {
            break;
        }
    }
    println!();

    let album = spotify.api.album("4muEF5biWb506ZojGMfHb7", Market::US).await?; // Ado ~ Kyougen
    println!("{:?} ~ {} by {} (\x1b]8;;{}\x1b\\Cover\x1b]8;;\x1b\\)", album.album_type, album.name, album.artists[0].name, album.images.first().unwrap().url);
    println!();

    println!("[Albums]");
    let albums = [
        "2tGokYNjX87AAodtbLBYuf", // Ado ~ Utattemita
        "7Ixqxq13tWhrbnIabk3172", // Uta's Songs ~ One Piece Film Red
    ];
    for album in spotify.api.albums(albums, Market::US).await? {
        println!(" - {:?} ~ {} by {} (\x1b]8;;{}\x1b\\Cover\x1b]8;;\x1b\\)", album.album_type, album.name, album.artists[0].name, album.images.first().unwrap().url);
    }
    println!();

    println!("[Tracks: Ado ~ Utattemita]");
    let mut tracks = spotify.api.album_tracks::<5, _, _>("2tGokYNjX87AAodtbLBYuf", Market::US)?;
    while let Some(page) = tracks.next().await? {
        for track in page.items {
            println!(" - {:0>2}:{:0>2} {}", track.duration.num_minutes(), track.duration.num_seconds() % 60, track.name);
        }
    }
    println!();

    println!("[Saved Albums]");
    let mut tracks = spotify.api.saved_albums::<5, _>(Market::US)?;
    while let Some(page) = tracks.next().await? {
        for saved_album in page.items {
            println!(" - {}   {}", saved_album.added_at.format("%l-%M %P %b %e, %Y"), saved_album.album.name);
        }
    }
    println!();


    println!("Saving albums [Ado ~ Utattemita, Ado ~ Uta's Songs]");
    spotify.api.save_albums(albums).await?;
    println!("[Saved]");
    for (album, added) in ["Ado ~ Utattemita", "Ado ~ Uta's Songs"].iter().zip(spotify.api.check_saved_albums(albums).await?) {
        println!(" - {}: {}", album, added);
    }
    let _ = dialoguer::Confirm::new()
        .with_prompt("Continue to remove albums [Ado ~ Utattemita, Ado ~ Uta's Songs]?")
        .interact();
    println!("Removing albums [Ado ~ Utattemita, Ado ~ Uta's Songs]");
    spotify.api.remove_saved_albums(albums).await?;
    println!();
    Ok(())
}
