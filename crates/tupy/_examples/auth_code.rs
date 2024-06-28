use tupy::api::{
    auth::OAuth,
    flow::{AuthCode, Credentials},
    scopes, Spotify, UserApi,
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

    let spotify = Spotify::<AuthCode>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let up = spotify.api.current_user_profile().await?;
    println!("[User]\n{:?}: {}\n", up.display_name, up.id);

    Ok(())
}
