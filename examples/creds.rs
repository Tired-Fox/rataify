use tupy::{
    api::{
        auth::OAuth, flow::{Credentials, Creds}, response::ReleaseDate, PublicApi, Spotify 
    }, Pagination,
    //AsyncIter
};

fn format_release(release: &ReleaseDate) -> String {
    match release {
        ReleaseDate::Day(day) => day.format("%b %d, %Y"),
        ReleaseDate::Month(day) => day.format("%d, %Y"),
        ReleaseDate::Year(day) => day.format("%Y"),
    }.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Creds>::new(
        Credentials::from_env().unwrap(),
        oauth,
        "tupy"
    ).await?;

    let mut new_releases = spotify.api.new_releases::<1>()?;
    if let Some(page) = new_releases.next().await? {
        println!("Page [{}]", new_releases.page());
        for item in page.items {
            println!(" - {}   {}", format_release(&item.release), item.name);
        }
    }
    if let Some(page) = new_releases.next().await? {
        println!("Page [{}]", new_releases.page());
        for item in page.items {
            println!(" - {}   {}", format_release(&item.release), item.name);
        }
    }
    if let Some(page) = new_releases.prev().await? {
        println!("Page [{}]", new_releases.page());
        for item in page.items {
            println!(" - {}   {}", format_release(&item.release), item.name);
        }
    }
    Ok(())
}
