use std::{collections::HashMap, fmt::Display, future::Future};

use crate::{pares, Error};

use super::{
    flow::AuthFlow, request, response::{NewReleases, Paginated}, SpotifyResponse, API_BASE_URL
};

pub trait PublicApi: AuthFlow {
    /// Get a list of new album releases featured in Spotify
    /// (shown, for example, on a Spotify player’s “Browse” tab).
    fn get_new_releases<const N: usize>(
        &self,
    ) -> Result<Paginated<NewReleases, HashMap<String, NewReleases>, Self, N>, Error> {
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/browse/new-releases?limit={}&offset={}",
                API_BASE_URL,
                N,
                0,
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

    /// Check to see if the current user is following a specified playlist.
    ///
    /// # Arguments
    /// - `playlist_id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the playlist.
    fn check_follow_playlist<D: Display>(&self, playlist_id: D) -> impl Future<Output = Result<bool, Error>> {
        async move {
            let SpotifyResponse { body,.. } = request::get!("playlists/{playlist_id}/followers/contains")
                .send(self.token().await?)
                .await?;
            let values: Vec<bool> = pares!(&body)?;
            Ok(*values.first().unwrap_or(&false))
        }
    }
}
