use tupy::{
    api::{
        auth::OAuth, flow::{Credentials, AuthCode}, request::TimeRange, response::{Artist, Track}, scopes, Spotify, UserApi
    },
    Pagination
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([scopes::USER_TOP_READ]).unwrap();

    let spotify = Spotify::<AuthCode>::new(
        Credentials::from_env().unwrap(),
        oauth,
        "tupy"
    ).await?;

    let mut top_items = spotify.api.user_top_items::<Track, 1>(TimeRange::Medium)?;
    while let Some(page) = top_items.next().await? {
        println!("Page {}", top_items.page() + 1);
        for item in page.items {
            println!("{}", item.name);
        }
        if top_items.page() >= 1 {
            break;
        }
    }
    Ok(())
}
