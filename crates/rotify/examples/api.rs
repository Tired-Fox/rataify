extern crate rotify;

use rotify::{auth::OAuth, Spotify, SpotifyRequest};

#[tokio::main]
async fn main() {
    let mut spotify = Spotify::new();

    // TODO: Throw errors up and catch them before they go all the way up
    //  handle no device by opening device select
    //  handle invalid token by refreshing
    //  handle all other known errors by showing error toast or dialog
    //  handle all other errors by throwing the rest of the way up crashing the app

    // tokio::time::sleep(Duration::from_secs(1)).await;


    let result = spotify
        .player()
        // .add_to_queue(UriType::Track, "1DeXr9ob3s3HQbq3ypvifn")
        // .next()
        .pause()
        .send()
        .await;
    println!("{result:#?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_works() {
        let oauth = OAuth::new();
    }
}
