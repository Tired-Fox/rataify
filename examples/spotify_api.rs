extern crate rataify;

use rataify::{
    scopes,
    spot::auth::{Credentials, OAuth},
    spot::{AdditionalTypes, Spotify, SpotifyRequest, SpotifyResponse},
};

#[tokio::main]
async fn main() {
    let oauth = OAuth::new(
        Credentials::from_env().unwrap(),
        scopes![user_read_private,],
    );

    let mut spotify = Spotify::new(oauth);

    // TODO: Throw errors up and catch them before they go all the way up
    //  handle no device by opening device select
    //  handle invalid token by refreshing
    //  handle all other known errors by showing error toast or dialog
    //  handle all other errors by throwing the rest of the way up crashing the app
    let response = spotify
        .player_state()
        .market("US")
        .additional_types([AdditionalTypes::Track, AdditionalTypes::Episode])
        .send()
        .await;
    println!("{response:?}");

    let response = spotify.devices().send().await;
    println!("{response:?}");

    let _ = spotify
        .transfer_playback()
        .device(response.unwrap().devices.first().unwrap().id.clone())
        .play(true)
        .send()
        .await;
}
