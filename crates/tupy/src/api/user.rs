use std::{collections::HashMap, fmt::Debug, future::Future};

use base64::Engine;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::{pares, Error, api::{Uri, scopes}};

use super::{
    flow::AuthFlow,
    request::{
        self, IntoDuration, IntoSpotifyId, OptionalSpotifyId, Play, PlaylistAction, PlaylistDetails, TimeRange, Timestamp, UriWrapper, SUPPORTED_ITEMS
    },
    response::{
        Device, FollowedArtists, IntoUserTopItemType, PagedPlaylists, Paginated, Playback,
        Playlist, PlaylistItems, Profile, Queue, RecentlyPlayed, Repeat, SavedAlbums,
        SavedAudiobooks, SavedEpisodes, SavedShows, SavedTracks, TopItems,
    },
    validate_scope, IntoSpotifyParam, SpotifyResponse, API_BASE_URL,
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
            let token = self.token();
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
        validate_scope(self.scopes(), [scopes::USER_TOP_READ])?;
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
            |c: TopItems<T>| c,
        ))
    }

    /// Get public profile information about a Spotify user.
    fn user_profile<I: IntoSpotifyId>(
        &self,
        user_id: I,
    ) -> impl Future<Output = Result<Profile, Error>> {
        async move {
            let token = self.token();
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
                [
                    scopes::PLAYLIST_MODIFY_PUBLIC,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;
            let token = self.token();
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
                [
                    scopes::PLAYLIST_MODIFY_PUBLIC,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;
            let token = self.token();
            request::delete!("playlists/{}/followers", playlist_id.into_spotify_id())
                .send(token)
                .await?;
            Ok(())
        }
    }

    /// Get the current user's followed artists.
    ///
    /// # Arguments
    /// <N> Is the number of items to return per page.
    ///
    /// # Scopes
    /// - `user-follow-read`: Access your followers and who you are following.
    fn followed_artists<const N: usize>(
        &self,
    ) -> Result<Paginated<FollowedArtists, HashMap<String, FollowedArtists>, Self, N>, Error> {
        validate_scope(self.scopes(), [scopes::USER_FOLLOW_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/me/following?type=artist&limit={N}",
                API_BASE_URL,
            )),
            None,
            |c: HashMap<String, FollowedArtists>| c.get("artists").unwrap().to_owned(),
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
            validate_scope(self.scopes(), [scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token();
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
            validate_scope(self.scopes(), [scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token();
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
            validate_scope(self.scopes(), [scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token();
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
            validate_scope(self.scopes(), [scopes::USER_FOLLOW_MODIFY])?;
            let token = self.token();
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
            validate_scope(self.scopes(), [scopes::USER_FOLLOW_READ])?;
            let SpotifyResponse { body, .. } = request::get!(
                "me/following/contains?type=artist&ids={}",
                ids.into_iter()
                    .map(|s| s.into_spotify_id())
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .send(self.token())
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
            validate_scope(self.scopes(), [scopes::USER_FOLLOW_READ])?;
            let SpotifyResponse { body, .. } = request::get!(
                "me/following/contains?type=user&ids={}",
                ids.into_iter()
                    .map(|s| s.into_spotify_id())
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .send(self.token())
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

        validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: SavedAlbums| c,
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
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            let token = self.token();
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
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            let token = self.token();
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
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;

            let ids = ids.into_iter()
                .map(|s| s.into_spotify_id())
                .collect::<Vec<_>>()
                .join(",");
            if ids.is_empty() {
                return Ok(vec![]);
            }
            let SpotifyResponse { body, .. } = request::get!("me/albums/contains")
                .param("ids", ids)
                .send(self.token())
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a list of the audiobooks saved in the current Spotify user's 'Your Music' library.
    ///
    /// # Arguments
    /// <N> Is the number of items to return per page.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn saved_audiobooks<const N: usize>(
        &self,
    ) -> Result<Paginated<SavedAudiobooks, SavedAudiobooks, Self, N>, Error> {
        validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(format!("{API_BASE_URL}/me/audiobooks?limit={N}")),
            None,
            |c: SavedAudiobooks| c,
        ))
    }

    /// Save one or more audiobooks to the current Spotify user's library.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn save_audiobooks<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            let ids = ids
                .into_iter()
                .map(|s| s.into_spotify_id())
                .collect::<Vec<String>>()
                .join(",");
            request::put!("me/audiobooks")
                .param("ids", ids)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Remove one or more audiobooks from the Spotify user's library.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn remove_saved_audiobooks<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::delete!("me/audiobooks")
                .param(
                    "ids",
                    ids.into_iter()
                        .map(|s| s.into_spotify_id())
                        .collect::<Vec<String>>()
                        .join(","),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Check if one or more audiobooks are already saved in the current Spotify user's library.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn check_saved_audiobooks<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;

            let ids = ids.into_iter()
                .map(|s| s.into_spotify_id())
                .collect::<Vec<_>>()
                .join(",");
            if ids.is_empty() {
                return Ok(vec![]);
            }
            let SpotifyResponse { body, .. } = request::get!("me/audiobooks/contains")
                .param("ids", ids)
                .send(self.token())
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a list of the episodes saved in the current Spotify user's library.
    ///
    /// # Arguments
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    /// - `user-read-playback-position` [Optional]: Read your position in content you have played.
    fn saved_episodes<const N: usize, M: IntoSpotifyParam>(
        &self,
        market: M,
    ) -> Result<Paginated<SavedEpisodes, SavedEpisodes, Self, N>, Error> {
        let mut url = format!("{API_BASE_URL}/me/episodes?limit={N}");

        if let Some(market) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", market));
        }

        validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: SavedEpisodes| c,
        ))
    }

    /// Save one or more episodes to the current user's library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the episodes. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn save_episodes<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::put!("me/episodes")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<_>>()
                    }}
                    .to_string(),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Remove one or more episodes from the current user's library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the episodes. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn remove_saved_episodes<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::delete!("me/episodes")
                .body(
                    json! {{
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<_>>()
                    }}
                    .to_string(),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Check if one or more episodes is already saved in the current Spotify user's 'Your Episodes' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the episodes. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn check_saved_episodes<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;

            let ids = ids.into_iter()
                        .map(|s| s.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(",");
            if ids.is_empty() {
                return Ok(vec![]);
            }

            let SpotifyResponse { body, .. } = request::get!("me/episodes/contains")
                .param("ids", ids)
                .send(self.token())
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a list of shows saved in the current Spotify user's library. Optional parameters can be used to limit the number of shows returned.
    ///
    /// # Arguments
    /// <N> Is the number of items to return per page.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn saved_shows<const N: usize>(
        &self,
    ) -> Result<Paginated<SavedShows, SavedShows, Self, N>, Error> {
        validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(format!("{API_BASE_URL}/me/shows?limit={N}")),
            None,
            |c: SavedShows| c,
        ))
    }

    /// Save one or more shows to current Spotify user's library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the shows. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn save_shows<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::put!("me/shows")
                .param(
                    "ids",
                    ids.into_iter()
                        .map(|s| s.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Delete one or more shows from current Spotify user's library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the shows. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn remove_saved_shows<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::delete!("me/shows")
                .param(
                    "ids",
                    ids.into_iter()
                        .map(|s| s.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Check if one or more shows is already saved in the current Spotify user's library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the shows. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn check_saved_shows<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;

            let ids = ids.into_iter()
                .map(|s| s.into_spotify_id())
                .collect::<Vec<_>>()
                .join(",");
            if ids.is_empty() {
                return Ok(vec![]);
            }
            let SpotifyResponse { body, .. } = request::get!("me/shows/contains")
                .param("ids", ids)
                .send(self.token())
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a list of the songs saved in the current Spotify user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn saved_tracks<const N: usize, M: IntoSpotifyParam>(
        &self,
        market: M,
    ) -> Result<Paginated<SavedTracks, SavedTracks, Self, N>, Error> {
        let mut url = format!("{API_BASE_URL}/me/tracks?limit={N}");

        if let Some(market) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", market));
        }

        validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: SavedTracks| c,
        ))
    }

    /// Save one or more tracks to the current user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the tracks. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn save_tracks<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::put!("me/tracks")
                .body(
                    json!({
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<_>>()
                    })
                    .to_string(),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Remove one or more tracks from the current user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the tracks. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-modify`: Manage your saved content.
    fn remove_saved_tracks<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_MODIFY])?;

            request::delete!("me/tracks")
                .body(
                    json!({
                        "ids": ids.into_iter().map(|s| s.into_spotify_id()).collect::<Vec<_>>()
                    })
                    .to_string(),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Check if one or more tracks is already saved in the current Spotify user's 'Your Music' library.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the tracks. Maximum: 50 IDs.
    ///
    /// # Scopes
    /// - `user-library-read`: Access your saved content.
    fn check_saved_tracks<S: IntoSpotifyId, I: IntoIterator<Item = S>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<bool>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_LIBRARY_READ])?;


            let ids = ids.into_iter()
                .map(|s| s.into_spotify_id())
                .collect::<Vec<_>>()
                .join(",");
            if ids.is_empty() {
                return Ok(vec![]);
            }
            let SpotifyResponse { body, .. } = request::get!("me/tracks/contains")
                .param("ids", ids)
                .send(self.token())
                .await?;

            Ok(pares!(&body)?)
        }
    }
    /// Get full details of the items of a playlist owned by a Spotify user.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page. Maximum: 50
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    /// - Spotify [content may not be used to train machine learning or AI models](https://developer.spotify.com/terms#section-iv-restrictions)
    fn playlist_items<const N: usize, I, M>(
        &self,
        id: I,
        market: M,
    ) -> Result<Paginated<PlaylistItems, PlaylistItems, Self, N>, Error>
    where
        I: IntoSpotifyId,
        M: IntoSpotifyParam,
    {
        let mut url = format!(
            "{API_BASE_URL}/playlists/{}/tracks?limit={N}&additional_types={SUPPORTED_ITEMS}",
            id.into_spotify_id()
        );

        if let Some(m) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", m));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: PlaylistItems| c,
        ))
    }

    /// Update the details of a playlist owned by a Spotify user.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `details`: The playlist details to update.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage your public playlists.
    /// - `playlist-modify-private`: Manage your private playlists.
    fn update_playlist_details<I: IntoSpotifyId>(
        &self,
        id: I,
        details: PlaylistDetails,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(
                self.scopes(),
                [
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;

            request::put!("playlists/{}", id.into_spotify_id())
                .body(serde_json::to_string(&details)?)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Either reorder or replace items in a playlist depending on the request's parameters.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `action`: The action to perform on the playlist.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage your public playlists.
    /// - `playlist-modify-private`: Manage your private playlists.
    fn update_playlist_items<I: IntoSpotifyId>(
        &self,
        id: I,
        action: PlaylistAction,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(
                self.scopes(),
                [
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;

            request::put!("playlists/{}/tracks", id.into_spotify_id())
                .body(serde_json::to_string(&action)?)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Add one or more items to a user's playlist.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `uris`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the items to add.
    /// - `at`: The position to insert the items, a zero-based index. For example, to insert the items in the first position: `at=0`. If omitted, the items will be appended to the playlist.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage your public playlists.
    /// - `playlist-modify-private`: Manage your private playlists.
    fn add_items<I, U>(
        &self,
        id: I,
        uris: U,
        at: Option<usize>,
    ) -> impl Future<Output = Result<String, Error>>
    where
        I: IntoSpotifyId,
        U: IntoIterator<Item = Uri>,
    {
        let mut body: HashMap<&str, serde_json::Value> = HashMap::new();
        body.insert(
            "uris",
            uris.into_iter()
                .map(|u| u.to_string().into())
                .collect::<Vec<serde_json::Value>>()
                .into(),
        );
        if let Some(at) = at {
            body.insert("position", at.into());
        }

        async move {
            validate_scope(
                self.scopes(),
                [
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;

            let SpotifyResponse { body, .. } =
                request::post!("playlists/{}/tracks", id.into_spotify_id())
                    .body(serde_json::to_string(&body)?)
                    .send(self.token())
                    .await?;

            let result: HashMap<String, String> = pares!(&body)?;
            Ok(result.get("snapshot_id").unwrap().to_owned())
        }
    }

    ///
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `uris`: A list of the [Spotify IDs](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the items to add.
    /// - `at`: The position to insert the items, a zero-based index. For example, to insert the items in the first position: `at=0`. If omitted, the items will be appended to the playlist.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage your public playlists.
    /// - `playlist-modify-private`: Manage your private playlists.
    fn remove_items<I, U>(&self, id: I, uris: U) -> impl Future<Output = Result<String, Error>>
    where
        I: IntoSpotifyId,
        U: IntoIterator<Item = Uri>,
    {
        let mut body: HashMap<&str, Vec<UriWrapper>> = HashMap::new();
        body.insert(
            "tracks",
            uris.into_iter()
                .map(|u| UriWrapper(u))
                .collect::<Vec<UriWrapper>>(),
        );

        async move {
            validate_scope(
                self.scopes(),
                [
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;

            let SpotifyResponse { body, .. } =
                request::delete!("playlists/{}/tracks", id.into_spotify_id())
                    .body(serde_json::to_string(&body)?)
                    .send(self.token())
                    .await?;

            let result: HashMap<String, String> = pares!(&body)?;
            Ok(result.get("snapshot_id").unwrap().to_owned())
        }
    }

    /// Get a list of the playlists owned or followed by the a Spotify user.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the user. Or if omitted, the current user.
    /// <N> Is the number of items to return per page. Maximum: 50
    ///
    /// # Scopes
    /// - `playlist-read-private` (Current and Other user): Access your private playlists.
    /// - `playlist-read-collaborative` (Other user): Access your collaborative playlists.
    fn playlists<const N: usize, I: OptionalSpotifyId>(
        &self,
        id: I,
    ) -> Result<Paginated<PagedPlaylists, PagedPlaylists, Self, N>, Error> {
        let id = id.optional_spotify_id();
        if let Some(id) = id {
            validate_scope(
                self.scopes(),
                [
                    scopes::PLAYLIST_READ_PRIVATE,
                    scopes::PLAYLIST_READ_COLLABORATIVE,
                ],
            )?;
            Ok(Paginated::new(
                self.clone(),
                Some(format!(
                    "{API_BASE_URL}/users/{}/playlists?limit={N}",
                    id.into_spotify_id()
                )),
                None,
                |c: PagedPlaylists| c,
            ))
        } else {
            validate_scope(self.scopes(), [scopes::PLAYLIST_READ_PRIVATE])?;
            Ok(Paginated::new(
                self.clone(),
                Some(format!("{API_BASE_URL}/me/playlists?limit={N}")),
                None,
                |c: PagedPlaylists| c,
            ))
        }
    }

    /// Create a playlist for a Spotify user. (The playlist will be empty until you add tracks.) Each user is generally limited to a maximum of 11000 playlists.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the user.
    /// - `details`: The playlist details to create.
    ///
    /// # Scopes
    /// - `playlist-modify-public`: Manage your public playlists.
    /// - `playlist-modify-private`: Manage your private playlists.
    fn create_playlist<I: IntoSpotifyId>(
        &self,
        id: I,
        details: PlaylistDetails,
    ) -> impl Future<Output = Result<Playlist, Error>> {
        async move {
            validate_scope(
                self.scopes(),
                [
                    scopes::PLAYLIST_MODIFY_PUBLIC,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                ],
            )?;

            let SpotifyResponse { body, .. } =
                request::post!("users/{}/playlists", id.into_spotify_id())
                    .body(serde_json::to_string(&details)?)
                    .send(self.token())
                    .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Replace the image used to represent a specific playlist.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `image`: The image to use as the playlist's new image. Any image in the jpeg format.
    /// Expects raw bytes.
    ///
    /// # Scopes
    /// - `ugc-image-upload`: Upload images to Spotify on your behalf.
    /// - `playlist-modify-public`: Manage your public playlists.
    /// - `playlist-modify-private`: Manage your private playlists.
    fn add_playlist_cover_image<I: IntoSpotifyId, D: AsRef<[u8]>>(
        &self,
        id: I,
        image: D,
    ) -> impl Future<Output = Result<(), Error>> {
        let image = base64::engine::general_purpose::STANDARD.encode(image);
        async move {
            if image.bytes().len() > (256 * 1024) {
                return Err(Error::InvalidArgument(
                    "image",
                    "Image size is too large. Max size is 256KB after base64 encoding".to_string(),
                ));
            }

            validate_scope(
                self.scopes(),
                [
                    scopes::UGC_IMAGE_UPLOAD,
                    scopes::PLAYLIST_MODIFY_PRIVATE,
                    scopes::PLAYLIST_MODIFY_PUBLIC,
                ],
            )?;

            request::put!("playlists/{}/images", id.into_spotify_id())
                .body(serde_json::to_string(&image)?)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    ///Get information about the user’s current playback state, including track or episode, progress, and active device.
    ///
    /// # Arguments
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Scopes
    /// - `user-read-playback-state`: Read your currently playing content and Spotify Connect devices information.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn playback_state<M: IntoSpotifyParam>(
        &self,
        market: M,
    ) -> impl Future<Output = Result<Option<Playback>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_READ_PLAYBACK_STATE])?;

            let token = self.token();
            let SpotifyResponse { body, status, .. } = request::get!("me/player")
                .param("market", market)
                .param("additional_types", SUPPORTED_ITEMS)
                .send(token)
                .await?;

            if status == StatusCode::NO_CONTENT {
                Ok(None)
            } else {
                Ok(Some(pares!(&body)?))
            }
        }
    }

    /// Transfer playback to a new device and optionally begin playback. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to which playback should be transferred.
    /// - `play`: If `true` the new device should start playing immediately.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    fn transfer_playback<I: IntoSpotifyId>(
        &self,
        id: I,
        play: bool,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player")
                .body(
                    json!({
                        "device_ids": [id.into_spotify_id()],
                        "play": play
                    })
                    .to_string(),
                )
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Get information about a user’s available Spotify Connect devices. Some device models are not supported and will not be listed in the API response.
    ///
    /// # Scopes
    /// - `user-read-playback-state`: Read your currently playing content and Spotify Connect devices information.
    fn devices(&self) -> impl Future<Output = Result<Vec<Device>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_READ_PLAYBACK_STATE])?;

            let SpotifyResponse { body, .. } = request::get!("me/player/devices")
                .send(self.token())
                .await?;

            let devices: HashMap<String, Vec<Device>> = pares!(&body)?;
            Ok(devices.get("devices").unwrap().to_owned())
        }
    }

    /// Get the object currently being played on the user's Spotify account.
    ///
    /// # Arguments
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Scopes
    /// - `user-read-currently-playing`: Read your currently playing content.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn currently_playing<M: IntoSpotifyParam>(
        &self,
        market: M,
    ) -> impl Future<Output = Result<Option<Playback>, Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_READ_CURRENTLY_PLAYING])?;

            let SpotifyResponse { body, .. } = request::get!("me/player/currently-playing")
                .param("market", market)
                .param("additional_types", SUPPORTED_ITEMS)
                .send(self.token())
                .await?;

            Ok(Some(pares!(&body)?))
        }
    }

    /// Start a new context or resume current playback on the user's active device. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    /// - `action`: The action to perform. Whether this is playing an artist, album, or playlist. Playing multiple tracks, episodes, or chapters. Or just resuming playback.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    fn play<I: IntoSpotifyParam>(
        &self,
        action: Play,
        id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player/play")
                .param("device_id", id)
                .body(serde_json::to_string(&action)?)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Pause playback on the user's account. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn pause<I: IntoSpotifyParam>(&self, id: I) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player/pause")
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Skips to next track in the user’s queue. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn next<I: IntoSpotifyParam>(&self, id: I) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::post!("me/player/next")
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Skips to previous track in the user’s queue. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn prev<I: IntoSpotifyParam>(&self, id: I) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::post!("me/player/previous")
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Seeks to the given position in the user’s currently playing track. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn seek<I: IntoSpotifyParam, P: IntoDuration>(
        &self,
        position: P,
        id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player/seek")
                .param("position_ms", position.into_duration().num_milliseconds())
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Set the repeat mode for the user's playback. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `repeat`: The new repeat mode
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    fn repeat<I: IntoSpotifyParam>(
        &self,
        repeat: Repeat,
        id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player/repeat")
                .param("state", repeat)
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    ///Set the volume for the user’s current playback device. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `volume`: The new volume, in the range 0..100
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    fn volume<I: IntoSpotifyParam>(
        &self,
        volume: u8,
        id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player/volume")
                .param("volume_percent", volume)
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Toggle shuffle on or off for user’s playback. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `shuffle`: The new shuffle mode
    /// - `id` [Optional]: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the device to target.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    fn shuffle<I: IntoSpotifyParam>(
        &self,
        shuffle: bool,
        id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::put!("me/player/shuffle")
                .param("state", shuffle)
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }

    /// Get tracks from the current user's recently played tracks. Note: Currently doesn't support podcast episodes.
    ///
    /// # Arguments
    /// - `timestamp`: A timestamp in ISO 8601 format: yyyy-MM-ddTHH:mm:ss. Use this parameter to specify the user’s playback position. If no timestamp is specified, the most recent position is assumed.
    /// <N> Is the number of items to return per page. Maximum: 50
    ///
    /// # Scopes
    /// - `user-read-recently-played`: Read a user’s recently played tracks.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn recently_played<const N: usize>(
        &self,
        timestamp: Timestamp,
    ) -> Result<Paginated<RecentlyPlayed, RecentlyPlayed, Self, N>, Error> {
        validate_scope(self.scopes(), [scopes::USER_READ_RECENTLY_PLAYED])?;

        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{API_BASE_URL}/me/player/recently-played?limit={N}&{}={}",
                timestamp.name(),
                timestamp.into_spotify_param().unwrap()
            )),
            None,
            |c: RecentlyPlayed| c,
        ))
    }

    /// Get the list of objects that make up the user's queue.
    ///
    /// # Scopes
    /// - `user-read-currently-playing`: Read your currently playing content.
    /// - `user-read-playback-state`: Read your currently playing content and Spotify Connect devices information.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn queue(&self) -> impl Future<Output = Result<Queue, Error>> {
        async move {
            validate_scope(
                self.scopes(),
                [
                    scopes::USER_READ_CURRENTLY_PLAYING,
                    scopes::USER_READ_PLAYBACK_STATE,
                ],
            )?;

            let SpotifyResponse { body, .. } = request::get!("me/player/queue")
                .send(self.token())
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Add an item to the end of the user's current playback queue. This API only works for users who have Spotify Premium. The order of execution is not guaranteed when you use this API with other Player API endpoints.
    ///
    /// # Arguments
    /// - `uri`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the item to add to the queue.
    ///
    /// # Scopes
    /// - `user-modify-playback-state`: Control playback on your Spotify clients and Spotify Connect devices.
    ///
    /// # Important Policy Notes
    /// - Streaming [aplication may not be commercial](https://developer.spotify.com/policy/#iv-streaming-and-commercial-use:~:text=Commercial%20use%20restrictions,Streaming%20SDA%20itself.)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Do not [synchronize Spotify content](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20synchronize%20any%20sound%20recordings%20with%20any%20visual%20media,.)
    /// - Spotify [content may not be broadcasted](https://developer.spotify.com/policy/#iii-some-prohibited-applications:~:text=Do%20not%20create%20any%20product%20or%20service%20which%20includes%20any%20non,several%20simultaneous%20listeners.)
    fn add_to_queue<I: IntoSpotifyParam>(
        &self,
        uri: Uri,
        id: I,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            validate_scope(self.scopes(), [scopes::USER_MODIFY_PLAYBACK_STATE])?;

            request::post!("me/player/queue")
                .param("uri", uri)
                .param("device_id", id)
                .send(self.token())
                .await?;

            Ok(())
        }
    }
}
