use tupy::api::{
    auth::OAuth,
    flow::{Credentials, Pkce},
    PublicApi, Spotify,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    println!("[Available Genres Seeds]");
    let genres = spotify.api.available_genre_seeds().await?;
    for (i, genre) in genres.iter().enumerate() {
        println!(" - {}", genre);

        if i >= 15 {
            println!(" - ...{} More...", genres.len() - i);
            break;
        }
    }
    Ok(())
}
