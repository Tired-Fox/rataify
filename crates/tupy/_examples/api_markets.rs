use tupy::api::{
    auth::OAuth,
    flow::{Credentials, Pkce},
    PublicApi, Spotify,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    println!("[Available Markets]");
    let markets = spotify.api.available_markets().await?;
    for (i, market) in markets.iter().enumerate() {
        println!(" - {}", market);

        if i >= 15 {
            println!(" - ...{} More...", markets.len() - i);
            break;
        }
    }
    Ok(())
}
