use tupy::{
    api::{
        auth::OAuth, flow::creds::{Credentials, Flow}, PublicApi, Spotify 
    }, Pagination,
    //AsyncIter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Flow>::new(
        Credentials::from_env().unwrap(),
        oauth,
        "tupy"
    ).await?;

    let mut new_releases = spotify.api.get_new_releases::<1>()?;
    if let Some((i, page)) = new_releases.next().await {
        println!("Page [{}]", i);
        for item in page.albums.items {
            println!("{}", item.name);
        }
    }
    if let Some((i, page)) = new_releases.next().await {
        println!("Page [{}]", i);
        for item in page.albums.items {
            println!("{}", item.name);
        }
    }
    if let Some((i, page)) = new_releases.prev().await {
        println!("Page [{}]", i);
        for item in page.albums.items {
            println!("{}", item.name);
        }
    }
    Ok(())
}
