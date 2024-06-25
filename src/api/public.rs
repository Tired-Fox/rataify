use std::{collections::HashMap, future::Future};

use crate::{pares, Error};

use super::{
    flow::AuthFlow,
    request::{self, IntoSpotifyId},
    response::{Album, AlbumTracks, NewReleases, Paginated},
    IntoSpotifyParam, SpotifyResponse, API_BASE_URL,
};

pub trait PublicApi: AuthFlow {
    /// Check to see if the current user is following a specified playlist.
    ///
    /// # Arguments
    /// - `playlist_id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the playlist.
    fn check_follow_playlist<I: IntoSpotifyId>(
        &self,
        playlist_id: I,
    ) -> impl Future<Output = Result<bool, Error>> {
        async move {
            let SpotifyResponse { body, .. } = request::get!(
                "playlists/{}/followers/contains",
                playlist_id.into_spotify_id()
            )
            .send(self.token().await?)
            .await?;
            let values: Vec<bool> = pares!(&body)?;
            Ok(*values.first().unwrap_or(&false))
        }
    }

    /// Get a list of new album releases featured in Spotify
    /// (shown, for example, on a Spotify player’s “Browse” tab).
    fn new_releases<const N: usize>(
        &self,
    ) -> Result<Paginated<NewReleases, HashMap<String, NewReleases>, Self, N>, Error> {
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/browse/new-releases?limit={}&offset={}",
                API_BASE_URL, N, 0,
            )),
            None,
            |c: HashMap<String, NewReleases>| {
                let new_releases = c.get("albums").unwrap().to_owned();
                let next = new_releases.next.clone();
                let previous = new_releases.previous.clone();
                (new_releases, previous, next)
            },
        ))
    }

    /// Get Spotify catalog information for a single album.
    ///
    /// # Arguments
    /// - `album_id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the album.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// **Important Policy Notes**:
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn album<I: IntoSpotifyId, M: IntoSpotifyParam>(
        &self,
        album_id: I,
        market: M,
    ) -> impl Future<Output = Result<Album, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } =
                request::get!("albums/{}", album_id.into_spotify_id())
                    .param("market", market)
                    .send(token)
                    .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for multiple albums identified by their Spotify IDs.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the albums. Maximum: 20 IDs.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// **Important Policy Notes**:
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn albums<D: IntoSpotifyId, M: IntoSpotifyParam, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
        market: M,
    ) -> impl Future<Output = Result<Vec<Album>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("albums")
                .param(
                    "ids",
                    ids.into_iter()
                        .map(|id| id.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .param("market", market)
                .send(token)
                .await?;

            let albums: HashMap<String, Vec<Album>> = pares!(&body)?;
            Ok(albums.get("albums").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information about an album’s tracks. Optional parameters can be used to limit the number of tracks returned.
    ///
    /// # Arguments
    /// - <N>: The maximum number of tracks to return per page.
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the album.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// **Important Policy Notes**:
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn album_tracks<const N: usize, D, M>(
        &self,
        id: D,
        market: M,
    ) -> Result<Paginated<AlbumTracks, AlbumTracks, Self, N>, Error>
    where
        D: IntoSpotifyId,
        M: IntoSpotifyParam,
    {
        let mut next = format!(
            "{}/albums/{}/tracks?limit={N}",
            API_BASE_URL,
            id.into_spotify_id(),
        );

        if let Some(m) = market.into_spotify_param() {
            next.push_str(&format!("&market={}", m));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(next),
            None,
            |c: AlbumTracks| {
                let next = c.next.clone();
                let previous = c.previous.clone();
                (c, previous, next)
            },
        ))
    }
}
