use tupy::{api::{
    auth::OAuth,
    flow::{Credentials, Pkce},
    PublicApi, Spotify,
}, Pagination};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let mut categories = spotify.api.browse_categories::<5, _>(None)?;
    while let Some(page) = categories.next().await? {
        for category in page.items {
            println!("{}", category.name);
        }

        if categories.page() >= 1 {
            break;
        }
    }

    println!("{:#?}", spotify.api.browse_category("0JQ5DAqbMKFGaKcChsSgUO", None).await?);

    Ok(())
}
