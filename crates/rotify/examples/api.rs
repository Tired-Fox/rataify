extern crate rotify;

use rotify::{AsyncIter, auth::OAuth, Spotify, SpotifyRequest};
use rotify::model::player::{Repeat, Track};

#[tokio::main]
async fn main() {
    let mut spotify = Spotify::new();

    // TODO: Throw errors up and catch them before they go all the way up
    //  handle no device by opening device select
    //  handle invalid token by refreshing
    //  handle all other known errors by showing error toast or dialog
    //  handle all other errors by throwing the rest of the way up crashing the app

    // tokio::time::sleep(Duration::from_secs(1)).await;

    let playlist = "37i9dQZF1DX0b1hHYQtJjp";
    let id = "d0nko8z8jy6gcbkclk4lgik6d";

    let playback = spotify
        .player()
        .playback()
        .send()
        .await
        .expect("Failed to get spotify playback state");

    println!("{:#?}", playback);

    // let mut track_iter = spotify
    //     .tracks()
    //     .get_saved_tracks()
    //     .iter();
    //
    // let mut count = 0;
    // while let Some(Ok(next_track)) = track_iter.next().await {
    //     for item in next_track.items.iter() {
    //         count += 1;
    //         println!("{count}. {:#?}", item.track.name);
    //     }
    //
    //     if count >= 100 {
    //         break;
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_works() {
        let oauth = OAuth::new();
    }
}
