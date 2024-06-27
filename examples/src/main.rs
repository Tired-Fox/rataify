use tupy::api::{flow::{Pkce, Credentials}, UserApi, auth::OAuth, Spotify, scopes};
use api_examples::util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let oauth = OAuth::from_env([
        scopes::USER_READ_PLAYBACK_STATE
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(
        Credentials::from_env().unwrap(), 
        oauth,
        "tupy"
    )?;

    util::refresh_token(&spotify.api).await?;

    println!("{:#?}", spotify.api.playback_state(None).await?);
    Ok(())
}
