use rspotify::{clients::BaseClient, model::{SearchResult, SearchType, SimplifiedPlaylist}, AuthCodePkceSpotify};

use crate::Error;

pub async fn daily_mixes(spotify: &AuthCodePkceSpotify) -> Result<Vec<SimplifiedPlaylist>, Error> {
    match spotify
        .search(
            "Daily Mix",
            SearchType::Playlist,
            None,
            None,
            Some(12),
            None,
        )
        .await
    {
        Ok(SearchResult::Playlists(page)) => {
            let mut mixes: Vec<SimplifiedPlaylist> = page.items.into_iter().filter(|p| {
                p.owner.display_name.as_deref() == Some("Spotify")
                && p.name.as_str().starts_with("Daily Mix ")
            }).collect();

            mixes.sort_by(|first, second| first.name.cmp(&second.name));

            Ok(mixes)
        },
        Ok(_) => Err(Error::custom("expected playlists form spotify search api")),
        Err(err) => Err(Error::from(err))
    }
}

pub async fn release_radar(spotify: &AuthCodePkceSpotify) -> Result<Option<SimplifiedPlaylist>, Error> {
    match spotify
        .search(
            "release radar",
            SearchType::Playlist,
            None,
            None,
            Some(2),
            None,
        )
        .await
    {
        Ok(SearchResult::Playlists(page)) => {
            Ok(page.items.into_iter().filter(|p| {
                p.owner.display_name.as_deref() == Some("Spotify")
            }).find(|p| p.name.as_str() == "Release Radar" ))
        },
        Ok(_) => Err(Error::custom("expected playlists form spotify search api")),
        Err(err) => Err(Error::from(err))
    }
}

pub async fn discover_weekly(spotify: &AuthCodePkceSpotify) -> Result<Option<SimplifiedPlaylist>, Error> {
    match spotify
        .search(
            "discover weekly",
            SearchType::Playlist,
            None,
            None,
            Some(2),
            None,
        )
        .await
    {
        Ok(SearchResult::Playlists(page)) => {
            Ok(page.items.into_iter().filter(|p| {
                p.owner.display_name.as_deref() == Some("Spotify")
            }).find(|p| p.name.as_str() == "Discover Weekly" ))
        },
        Ok(_) => Err(Error::custom("expected playlists form spotify search api")),
        Err(err) => Err(Error::from(err))
    }
}
