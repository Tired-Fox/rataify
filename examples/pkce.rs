use tupy::{
    api::{
        auth::OAuth,
        flow::{Pkce, Credentials},
        scopes,
        Spotify, UserApi
    },
    //AsyncIter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let oauth = OAuth::from_env([scopes::USER_TOP_READ]).unwrap();

    let spotify = Spotify::<Pkce>::new(
        Credentials::from_env().unwrap(),
        oauth,
        "tupy"
    ).await?;

    println!("{:#?}", spotify.api.get_current_user_profile().await?);
    Ok(())
}
