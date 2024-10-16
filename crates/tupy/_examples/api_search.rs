use std::env::args;

use chrono::Datelike;
use tupy::{
    api::{
        auth::OAuth,
        flow::{Credentials, Pkce},
        request::{Query, QueryTag, SearchType},
        PublicApi, Spotify,
    },
    Pagination,
};

fn parse_query() -> Vec<Query> {
    let mut query = Vec::new();
    for arg in args().skip(1) {
        if arg.contains(':') {
            match arg.split_once(':').unwrap() {
                ("artist", artist) => query.push(Query::artist(artist)),
                ("genre", genre) => query.push(Query::genre(genre)),
                ("track", track) => query.push(Query::track(track)),
                ("album", album) => query.push(Query::album(album)),
                ("upc", upc) => query.push(Query::upc(upc)),
                ("isrc", isrc) => query.push(Query::isrc(isrc)),
                ("tag", "new") => query.push(Query::tag(QueryTag::New)),
                ("tag", "hipster") => query.push(Query::tag(QueryTag::Hipster)),
                _ => eprintln!("Warning: Unknown query {:?}", arg),
            }
        } else {
            query.push(Query::text(arg));
        }
    }

    if query.is_empty() {
        eprintln!("Warning: No query provided, using default query 'artist:ado genre:j-pop'");
        query.push(Query::artist("ado"));
        query.push(Query::genre("j-pop"));
    }

    query
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1mSpotify Search:\x1b[0m");
    println!("  \x1b[33mUse\x1b[39m `<\x1b[32mtext\x1b[39m>`, `\x1b[35martist:\x1b[39m<\x1b[32mtext\x1b[39m>`, `\x1b[35mtrack:\x1b[39m<\x1b[32mtext\x1b[39m>`, `\x1b[35mgenre:\x1b[39m<\x1b[32mtext\x1b[39m>`, `\x1b[35malbum:\x1b[39m<\x1b[32mtext\x1b[39m>`,");
    println!("  `\x1b[35mupc:\x1b[39m<\x1b[32mtext\x1b[39m>`, `\x1b[35misrc:\x1b[39m<\x1b[32mtext\x1b[39m>`, `\x1b[35mtag:new\x1b[39m`, and `\x1b[35mtag:hipster\x1b[39m` to \x1b[36mcreate a query.\x1b[0m");
    println!();
    println!("  <\x1b[32mtext\x1b[39m>: \x1b[32mword\x1b[39m or \x1b[32m'word <word...>'\x1b[39m");
    println!();

    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let query = parse_query();

    println!("Searching... ({})", query.iter().map(|q| q.to_string()).collect::<Vec<_>>().join(" "));

    let mut search = spotify.api.search::<10, _>(
        query.as_slice(),
        SearchType::all(),
        None,
        true,
    )?;

    if let Some(tracks) = search.tracks() {
        println!("[Tracks]");
        while let Some(page) = tracks.next().await? {
            for track in page.items {
                println!(
                    " - {} ({}): \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    track.name,
                    track.album.release.as_ref().year(),
                    track.external_urls.spotify
                );
            }

            if tracks.page() >= 1 {
                println!(" - ...{} More...", tracks.total() - tracks.progress());
                break;
            }
        }
    }
    println!();

    if let Some(artists) = search.artists() {
        println!("[Artists]");
        while let Some(page) = artists.next().await? {
            for artist in page.items {
                println!(
                    " - {}: \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    artist.name, artist.external_urls.spotify
                );
            }
            if artists.page() >= 1 {
                println!(" - ...{} More...", artists.total() - artists.progress());
                break;
            }
        }
    }
    println!();

    if let Some(albums) = search.albums() {
        println!("[Albums]");
        while let Some(page) = albums.next().await? {
            for album in page.items {
                println!(
                    " - {} ({}): \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    album.name,
                    album.release.as_ref().year(),
                    album.external_urls.spotify
                );
            }
            if albums.page() >= 1 {
                println!(" - ...{} More...", albums.total() - albums.progress());
                break;
            }
        }
    }
    println!();

    if let Some(playlists) = search.playlists() {
        println!("[Playlists]");
        while let Some(page) = playlists.next().await? {
            for playlist in page.items {
                println!(
                    " - {}: \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    playlist.name,
                    playlist.external_urls.spotify
                );
            }
            if playlists.page() >= 1 {
                println!(" - ...{} More...", playlists.total() - playlists.progress());
                break;
            }
        }
    }
    println!();

    if let Some(shows) = search.shows() {
        println!("[Shows]");
        while let Some(page) = shows.next().await? {
            for show in page.items {
                println!(
                    " - {}: \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    show.name,
                    show.external_urls.spotify
                );
            }
            if shows.page() >= 1 {
                println!(" - ...{} More...", shows.total() - shows.progress());
                break;
            }
        }
    }
    println!();

    if let Some(episodes) = search.episodes() {
        println!("[Episodes]");
        while let Some(page) = episodes.next().await? {
            for episode in page.items {
                println!(
                    " - {}: \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    episode.name,
                    episode.external_urls.spotify
                );
            }
            if episodes.page() >= 1 {
                println!(" - ...{} More...", episodes.total() - episodes.progress());
                break;
            }
        }
    }
    println!();

    if let Some(audiobooks) = search.audiobooks() {
        println!("[Audiobooks]");
        while let Some(page) = audiobooks.next().await? {
            for audiobook in page.items {
                println!(
                    " - {}: \x1b]8;;{}\x1b\\Link\x1b]8;;\x1b\\",
                    audiobook.name,
                    audiobook.external_urls.spotify
                );
            }
            if audiobooks.page() >= 1 {
                println!(" - ...{} More...", audiobooks.total() - audiobooks.progress());
                break;
            }
        }
    }
    println!();
    Ok(())
}
