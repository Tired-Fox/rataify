use tupy::{
    api::{
        auth::OAuth, flow::{Credentials, Creds}, PublicApi, Spotify 
    }, Pagination,
    //AsyncIter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Creds>::new(
        Credentials::from_env().unwrap(),
        oauth,
        "tupy"
    ).await?;

    let mut new_releases = spotify.api.get_new_releases::<1>()?;
    if let Some(page) = new_releases.next().await {
        println!("Page [{}]", new_releases.page());
        for item in page.albums.items {
            println!("{}", item.name);
        }
    }
    if let Some(page) = new_releases.next().await {
        println!("Page [{}]", new_releases.page());
        for item in page.albums.items {
            println!("{}", item.name);
        }
    }
    if let Some(page) = new_releases.prev().await {
        println!("Page [{}]", new_releases.page());
        for item in page.albums.items {
            println!("{}", item.name);
        }
    }
    Ok(())
}
