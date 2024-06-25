use std::{collections::HashMap, fmt::Debug, future::Future};

use serde::Deserialize;
use serde_json::json;

use crate::{pares, Error};

use super::{
    flow::AuthFlow,
    request::{self, IntoSpotifyId, TimeRange},
    response::{FollowedArtists, IntoUserTopItemType, Paginated, Profile, SavedAlbums, TopItems},
    scopes, validate_scope, IntoSpotifyParam, SpotifyResponse, API_BASE_URL,
};

pub trait UserApi: AuthFlow {
    /// Get detailed profile information about the current user (including the current user's username).
    ///
    /// # Scopes
    /// - `user-read-private` [optional]: Access to the `product`, `explicit_content`, and `country` fields
    /// - `user-read-email` [optional]: Access to the `email` field
    fn current_user_profile(&self) -> impl Future<Output = Result<Profile, Error>> {
        async {
            // Get the token: This will refresh it if needed
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("me").send(token).await?;
            Ok(pares!(&body)?)
        }
    }

    /// Get the current user's top artists or tracks based on calculated affinity.
    ///
    /// # Arguments
    /// - `time_range`: Over what time frame the affinities are computed. Valid values: long_term (calculated from ~1 year of data and including all new data as it becomes available), medium_term (approximately last 6 months), short_term (approximately last 4 weeks). Default: medium_term
    ///
    /// <N> Is the number of items to return per page.
    /// <T> Is the type of items to return. [Artist, Track]
    fn user_top_items<T, const N: usize>(
        &self,
        time_range: TimeRange,
    ) -> Result<Paginated<TopItems<T>, TopItems<T>, Self, N>, Error>
    where
        T: IntoUserTopItemType + Deserialize<'static> + Debug + Clone + PartialEq,
    {
        validate_scope(self.scopes(), &[scopes::USER_TOP_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/me/top/{}?time_range={}&limit={}&offset={}",
                API_BASE_URL,
                T::into_top_item_type(),
                time_range,
                N,
                0,
            )),
            None,
            |c: TopItems<T>| {
                let next = c.next.clone();
                let previous = c.previous.clone();
                (c, previous, next)
            },
        ))
    }

    /// Get public profile information about a Spotify user.
    fn user_profile<I: IntoSpotifyId>(
        &self,
        user_id: I,
    ) -> impl Future<Output = Result<Profile, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("users/{}", user_id.into_spotify_id())
                .send(token)
                .await?;
            Ok(pares!(&body)?)
        }
    }

    /// Add the current user as a follower of a playlist.
    ///
    /// # Arguments
    /// - `playlist_id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `public`: If true the playlist will be included in user's public playlists (added to profile), if false it will remain private.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage `public` playlists
    /// - `playlist-modify-private`: Manage `private` playlists
    fn follow_playlist<I: IntoSpotifyId>(
        &self,
        playlist_id: I,
        public: bool,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(
                self.scopes(),
                &[
                    scopes::PLAYLIST_MODIFY_PUBLIC,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;
            let token = self.token().await?;
            request::put!("playlists/{}/followers", playlist_id.into_spotify_id())
                .body(format!("{{\"public\":{}}}", public))
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Remove the current user as a follower of a playlist.
    ///
    /// # Arguments
    /// - `playlist_id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage `public` playlists
    /// - `playlist-modify-private`: Manage `private` playlists
    fn unfollow_playlist<I: IntoSpotifyId>(
        &self,
        playlist_id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(
                self.scopes(),
                &[
                    scopes::PLAYLIST_MODIFY_PUBLIC,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;
            let token = self.token().await?;
            request::delete!("playlists/{}/followers", playlist_id.into_spotify_id())
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Get the current user's followed artists.
    ///
    /// # Scopes
    /// - `user-follow-read`: Access your followers and who you are following.
    fn followed_artists<const N: usize>(
        &self,
    ) -> Result<Paginated<FollowedArtists, HashMap<String, FollowedArtists>, Self, N>, Error> {
        validate_scope(self.scopes(), &[scopes::USER_FOLLOW_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/me/following?type=artist&limit={N}",
                API_BASE_URL,
            )),
            None,
            |c: HashMap<String, FollowedArtists>| {
                let fa = c.get("artists").unwrap().to_owned();
                let next = fa.next.clone();
                (fa, None, next)
            },
        ))
    }

    /// Add the current user as a follower of one or more artists.
    ///
    /// # Arguments
    /// - `ids`: An array of the artist IDs. For example: {ids:["74ASZWbe4lXaubB36ztrGX", "08td7MxkoHQkXnWAYD8d6Q"]}. A maximum of 50 IDs can be sent in one request.
    ///
    ///
    /// # Scopes
    /// - `user-follow-modify`: Manage your saved content.
    fn follow_artists<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token().await?;
            request::put!("me/following?type=artist")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<String>>()
                    }}
                    .to_string(),
                )
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Remove the current user as a follower of one or more artists.
    ///
    /// # Arguments
    /// - `ids`: An array of the artist IDs. For example: {ids:["74ASZWbe4lXaubB36ztrGX", "08td7MxkoHQkXnWAYD8d6Q"]}. A maximum of 50 IDs can be sent in one request.
    ///
    /// # Scopes
    /// - `user-follow-modify`: Manage your saved content.
    fn unfollow_artists<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token().await?;
            request::delete!("me/following?type=artist")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<String>>()
                    }}
                    .to_string(),
                )
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Add the current user as a follower of one or more Spotify users.
    ///
    /// # Arguments
    /// - `ids`: An array of the user IDs. For example: {ids:["74ASZWbe4lXaubB36ztrGX", "08td7MxkoHQkXnWAYD8d6Q"]}. A maximum of 50 IDs can be sent in one request.
    ///
    ///
    /// # Scopes
    /// - `user-follow-modify`: Manage your saved content.
    fn follow_users<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token().await?;
            request::put!("me/following?type=user")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<String>>()
                    }}
                    .to_string(),
                )
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Remove the current user as a follower of one or more Spotify users.
    ///
    /// # Arguments
    /// - `ids`: An array of the user IDs. For example: {ids:["74ASZWbe4lXaubB36ztrGX", "08td7MxkoHQkXnWAYD8d6Q"]}. A maximum of 50 IDs can be sent in one request.
    ///
    /// # Scopes
    /// - `user-follow-modify`: Manage your saved content.
    fn unfollow_users<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token().await?;
            request::delete!("me/following?type=user")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<String>>()
                    }}
                    .to_string(),
                )
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Check to see if the current user is following one or more artists.
    ///
    /// # Arguments
    /// - `ids`: An array of the artist IDs. For example: {ids:["74ASZWbe4lXaubB36ztrGX", "08td7MxkoHQkXnWAYD8d6Q"]}. A maximum of 50 IDs can be sent in one request.
    ///
    /// # Scopes
    /// - `user-follow-read`: Access your followers and who you are following.
    fn check_follow_artists<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_FOLLOW_READ])?;
            let SpotifyResponse { body, .. } = request::get!(
                "me/following/contains?type=artist&ids={}",
                ids.into_iter()
                    .map(|s| s.into_spotify_id())
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .send(self.token().await?)
            .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Check to see if the current user is following one or more Spotify users.
    ///
    /// # Arguments
    /// - `ids`: An array of the user IDs. For example: {ids:["74ASZWbe4lXaubB36ztrGX", "08td7MxkoHQkXnWAYD8d6Q"]}. A maximum of 50 IDs can be sent in one request.
    ///
    /// # Scopes
    /// - `user-follow-read`: Access your followers and who you are following.
    fn check_follow_users<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_FOLLOW_READ])?;
            let SpotifyResponse { body, .. } = request::get!(
                "me/following/contains?type=user&ids={}",
                ids.into_iter()
                    .map(|s| s.into_spotify_id())
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .send(self.token().await?)
            .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a list of the albums saved in the current Spotify user's 'Your Music' library.
    ///
    /// # Arguments
    /// - <N> Is the number of items to return per page.
    /// - `market`: An ISO 3166-1 alpha-2 country code. Provide this parameter if you want to apply [Track Relinking](https://developer.spotify.com/documentation/general/guides/track-relinking-guide/).
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn saved_albums<const N: usize, M: IntoSpotifyParam>(
        &self,
        market: M,
    ) -> Result<Paginated<SavedAlbums, SavedAlbums, Self, N>, Error> {
        let mut url = format!("{}/me/albums?limit={N}", API_BASE_URL,);

        if let Some(market) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", market));
        }

        validate_scope(self.scopes(), &[scopes::USER_LIBRARY_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: SavedAlbums| {
                let next = c.next.clone();
                let previous = c.previous.clone();
                (c, previous, next)
            },
        ))
    }

    /// Save one or more albums to the current user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the albums. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn save_albums<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_LIBRARY_MODIFY])?;

            let token = self.token().await?;
            request::put!("me/albums")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<String>>()
                    }}
                    .to_string(),
                )
                .send(token)
                .await?;

            Ok(())
        }
    }

    /// Remove one or more albums from the current user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the albums. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn remove_saved_albums<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_LIBRARY_MODIFY])?;

            let token = self.token().await?;
            request::delete!("me/albums")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<String>>()
                    }}
                    .to_string(),
                )
                .send(token)
                .await?;

            Ok(())
        }
    }

    /// Check if one or more albums is already saved in the current Spotify user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the albums. Maximum: 20 IDs.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn check_saved_albums<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), &[scopes::USER_LIBRARY_READ])?;

            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("me/albums/contains")
                .param(
                    "ids",
                    ids.into_iter()
                        .map(|s| s.into_spotify_id())
                        .collect::<Vec<String>>()
                        .join(","),
                )
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }
}
