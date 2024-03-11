extern crate rotify;

use rotify::{AsyncIter, auth::OAuth, Spotify, SpotifyRequest};
use rotify::model::playback::Track;

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
    println!(
        "{:#?}",
        spotify.users()
            .unfollow_playlist(playlist)
            .send()
            .await
    );

    println!(
        "{:#?}",
        spotify.users()
            .follow_playlist(playlist)
            .send()
            .await
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_works() {
        let oauth = OAuth::new();
    }
}
