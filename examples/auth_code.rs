use tupy::api::{
    auth::OAuth,
    flow::{Credentials, AuthCode},
    scopes, Spotify, UserApi,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([scopes::USER_TOP_READ]).unwrap();

    let spotify = Spotify::<AuthCode>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    println!("{:#?}", spotify.api.get_current_user_profile().await?);
    Ok(())
}
