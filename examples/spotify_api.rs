extern crate rataify;

// use std::time::Duration;
use rataify::{
    scopes,
    spot::{Spotify, SpotifyRequest},
    spot::auth::{Credentials, OAuth},
};
use rataify::spot::model::UriType;

#[tokio::main]
async fn main() {
    let oauth = OAuth::new(
        Credentials::from_env().unwrap(),
        scopes![
            user_read_private,
            user_read_recently_played,
            user_read_playback_state,
            user_modify_playback_state,
        ],
    );

    let mut spotify = Spotify::new(oauth);

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
