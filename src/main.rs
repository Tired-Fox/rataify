use tupy::api::{
    auth::{AuthCode, Credentials, OAuth}, Spotify, UserApi
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for (i, step) in AuthCode::steps().iter().enumerate() {
        println!("{}. {}", i+1, step);
    }

    // TODO: Other auth flows/methods
    let spotify = Spotify::<AuthCode>::new(
        Credentials::from_env().unwrap(),
        OAuth::from_env().unwrap(),
        "tupy"
    ).await?;

    println!("{:#?}", spotify.api.current_user_profile().await?);
    Ok(())
}
