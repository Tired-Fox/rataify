#![allow(unused_imports)]

use rataify::Error;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{SearchResult, SearchType, SimplifiedPlaylist},
    scopes, AuthCodePkceSpotify, Credentials, OAuth,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spotify = AuthCodePkceSpotify::with_config(
        Credentials::from_env()
            .ok_or("failed to parse spotify Credentials from environment variables")?,
        OAuth::from_env(scopes!["user-read-playback-state"])
            .ok_or("failed to parse spotify OAuth from environment variables")?,
        rspotify::Config {
            cache_path: dirs::cache_dir()
                .unwrap()
                .join("rataify")
                .join("token.json"),
            token_cached: true,
            token_refreshing: true,
            ..Default::default()
        },
    );

    let url = spotify.get_authorize_url(None)?;
    spotify.prompt_for_token(url.as_str()).await?;

    // if let SearchResult::Playlists(page) = spotify
    //     .search(
    //         "Daily Mix",
    //         SearchType::Playlist,
    //         None,
    //         None,
    //         Some(12),
    //         None,
    //     )
    //     .await?
    // {
    //     for item in page.items {
    //         println!("{} @ {:?}", item.name, item.owner.display_name);
    //     }
    // }
    
    println!("{:#?}", rataify::api::release_discover(&spotify).await);
    println!("{:#?}", rataify::api::daily_mixes(&spotify).await);


    Ok(())
}
