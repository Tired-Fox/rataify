use tupy::{
    api::{
        auth::OAuth, flow::auth::{Credentials, Flow}, request::TimeRange, response::{Artist, Track}, scopes, Spotify, UserApi
    },
    Pagination
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([scopes::USER_TOP_READ]).unwrap();

    let spotify = Spotify::<Flow>::new(
        Credentials::from_env().unwrap(),
        oauth,
        "tupy"
    ).await?;

    let mut top_items = spotify.api.get_user_top_items::<Artist, 1>(TimeRange::Medium)?;
    while let Some((i, page)) = top_items.next().await {
        println!("Page {}", i + 1);
        for item in page.items {
            println!("{}", item.name);
        }
        if i >= 1 {
            break;
        }
    }
    Ok(())
}
