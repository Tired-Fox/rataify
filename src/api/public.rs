use std::{collections::HashMap, future::Future};

use crate::{pares, Error};

use super::{
    flow::AuthFlow,
    request::{self, IncludeGroup, IntoSpotifyId},
    response::{Album, AlbumTracks, Artist, ArtistAlbums, Audiobook, Categories, Category, Chapter, Chapters, Episode, NewReleases, Paginated, Track},
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
                if new_releases.items.is_empty() || (new_releases.offset + new_releases.limit >= new_releases.total) {
                    (new_releases, previous, None)
                } else {
                    (new_releases, previous, next)
                }
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
                if c.items.is_empty() || (c.offset + c.limit >= c.total) {
                    (c, previous, None)
                } else {
                    (c, previous, next)
                }
            },
        ))
    }

    /// Get Spotify catalog information for a single artist identified by their unique Spotify ID.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the artist.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn artist<I: IntoSpotifyId>(&self, id: I) -> impl Future<Output = Result<Artist, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("artists/{}", id.into_spotify_id())
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for several artists based on their Spotify IDs.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the artists. Maximum: 100 IDs.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn artists<D: IntoSpotifyId, I: IntoIterator<Item=D>>(&self, ids: I) -> impl Future<Output = Result<Vec<Artist>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("artists")
                .param("ids", ids.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .send(token)
                .await?;

            let artists: HashMap<String, Vec<Artist>> = pares!(&body)?;
            Ok(artists.get("artists").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information about an artist's albums.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the artist.
    /// - `include_groups`: A of filters to apply to the response. If not supplied, all groups will be returned.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn artist_albums<const N: usize, I, M>(&self, id: I, market: M, include_groups: &[IncludeGroup]) -> Result<Paginated<ArtistAlbums, ArtistAlbums, Self, N>, Error>
        where
            M: IntoSpotifyParam,
            I: IntoSpotifyId,
    {
        let mut url = format!(
            "{}/artists/{}/albums?limit={N}",
            API_BASE_URL,
            id.into_spotify_id(),
        );

        if let Some(m) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", m));
        }

        if !include_groups.is_empty() {
            url.push_str(&format!("&include_groups={}", include_groups.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",")));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: ArtistAlbums| {
                let next = c.next.clone();
                let previous = c.previous.clone();
                if c.items.is_empty() || (c.offset + c.limit >= c.total) {
                    (c, previous, None)
                } else {
                    (c, previous, next)
                }
            },
        ))
    }

    /// Get Spotify catalog information about an artist's top tracks by country.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the artist.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn artist_top_tracks<I, M>(&self, id: I, market: M) -> impl Future<Output = Result<Vec<Track>, Error>>
        where
            I: IntoSpotifyId,
            M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("artists/{}/top-tracks", id.into_spotify_id())
                .param("market", market)
                .send(token)
                .await?;

            let tracks: HashMap<String, Vec<Track>> = pares!(&body)?;
            Ok(tracks.get("tracks").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information about artists similar to a given artist. Similarity is based on analysis of the Spotify community's listening history.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the artist.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn related_artists<I: IntoSpotifyId>(&self, id: I) -> impl Future<Output = Result<Vec<Artist>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("artists/{}/related-artists", id.into_spotify_id())
                .send(token)
                .await?;

            let artists: HashMap<String, Vec<Artist>> = pares!(&body)?;
            Ok(artists.get("artists").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information for a single audiobook. Audiobooks are only available within the US, UK, Canada, Ireland, New Zealand and Australia markets.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the audiobook.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn audiobook<I: IntoSpotifyId, M: IntoSpotifyParam>(&self, id: I, market: M) -> impl Future<Output = Result<Audiobook, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("audiobooks/{}", id.into_spotify_id())
                .param("market", market)
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for several audiobooks identified by their Spotify IDs. Audiobooks are only available within the US, UK, Canada, Ireland, New Zealand and Australia markets.
    ///
    /// # Arguments
    /// - `ids`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the audiobooks.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn audiobooks<D: IntoSpotifyId, I: IntoIterator<Item=D>, M: IntoSpotifyParam>(&self, ids: I, market: M) -> impl Future<Output = Result<Vec<Audiobook>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("audiobooks")
                .param("ids", ids.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .param("market", market)
                .send(token)
                .await?;

            let audiobooks: HashMap<String, Vec<Audiobook>> = pares!(&body)?;
            Ok(audiobooks.get("audiobooks").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information about an audiobook's chapters. Audiobooks are only available within the US, UK, Canada, Ireland, New Zealand and Australia markets.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the audiobook.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn audiobook_chapters<const N: usize, I: IntoSpotifyId, M: IntoSpotifyParam>(&self, id: I, market: M) -> Result<Paginated<Chapters, Chapters, Self, N>, Error> {
        let mut url = format!("{API_BASE_URL}/audiobooks/{}/chapters?limit={N}", id.into_spotify_id());

        if let Some(m) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", m));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: Chapters| {
                let next = c.next.clone();
                let previous = c.previous.clone();
                if c.items.is_empty() || (c.offset + c.limit >= c.total) {
                    (c, previous, None)
                } else {
                    (c, previous, next)
                }
            },
        ))
    }

    /// Get a list of categories used to tag items in Spotify (on, for example, the Spotify player’s “Browse” tab).
    ///
    /// # Arguments
    /// - `locale`: The desired language, consisting of a lowercase ISO 639-1 language code and an uppercase ISO 3166-1 alpha-2 country code, joined by an underscore.
    fn browse_categories<const N: usize, L: IntoSpotifyParam>(&self, locale: L) -> Result<Paginated<Categories, HashMap<String, Categories>, Self, N>, Error> {
        let mut url = format!("{API_BASE_URL}/browse/categories?limit={N}");

        if let Some(locale) = locale.into_spotify_param() {
            url.push_str(&format!("&locale={}", locale));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: HashMap<String, Categories>| {
                let c = c.get("categories").unwrap().to_owned();
                let next = c.next.clone();
                let previous = c.previous.clone();
                if c.items.is_empty() || (c.offset + c.limit >= c.total) {
                    (c, previous, None)
                } else {
                    (c, previous, next)
                }
            },
        ))
    }

    /// Get a single category used to tag items in Spotify (on, for example, the Spotify player’s “Browse” tab).
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the category.
    /// - `locale`: The desired language, consisting of a lowercase ISO 639-1 language code and an uppercase ISO 3166-1 alpha-2 country code, joined by an underscore.
    fn browse_category<I: IntoSpotifyId, L: IntoSpotifyParam>(&self, id: I, locale: L) -> impl Future<Output = Result<Category, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("browse/categories/{}", id.into_spotify_id())
                .param("locale", locale)
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }
    
    /// Get Spotify catalog information for a single audiobook chapter. Chapters are only available within the US, UK, Canada, Ireland, New Zealand and Australia markets.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn chapter<I: IntoSpotifyId, M: IntoSpotifyParam>(&self, id: I, market: M) -> impl Future<Output = Result<Chapter, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("chapters/{}", id.into_spotify_id())
                .param("market", market)
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for several audiobook chapters identified by their Spotify IDs. Chapters are only available within the US, UK, Canada, Ireland, New Zealand and Australia markets.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn chapters<D, I, M>(&self, id: I, market: M) -> impl Future<Output = Result<Vec<Chapter>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("chapters")
                .param("ids", id.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .param("market", market)
                .send(token)
                .await?;

            let chapters: HashMap<String, Vec<Chapter>> = pares!(&body)?;
            Ok(chapters.get("chapters").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information for a single episode identified by its unique Spotify ID.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the episode.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Scopes
    /// - `user-read-playback-position` [Optional]: Read your position in content you have played.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn episode<I: IntoSpotifyId, M: IntoSpotifyParam>(&self, id: I, market: M) -> impl Future<Output = Result<Episode, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("episodes/{}", id.into_spotify_id())
                .param("market", market)
                .send(token)
                .await?;
            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for a single episode identified by its unique Spotify ID.
    ///
    /// # Arguments
    /// - `ids`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the episodes.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Scopes
    /// - `user-read-playback-position` [Optional]: Read your position in content you have played.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn episodes<D, I, M>(&self, id: I, market: M) -> impl Future<Output = Result<Vec<Episode>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("episodes")
                .param("ids", id.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .param("market", market)
                .send(token)
                .await?;

            let episodes: HashMap<String, Vec<Episode>> = pares!(&body)?;
            Ok(episodes.get("episodes").unwrap().to_owned())
        }
    }

    /// Retrieve a list of available genres seed parameter values for [recommendations](https://developer.spotify.com/documentation/web-api/reference/get-recommendations).
    fn available_genre_seeds(&self) -> impl Future<Output = Result<Vec<String>, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("recommendations/available-genre-seeds")
                .send(token)
                .await?;

            let seeds: HashMap<String, Vec<String>> = pares!(&body)?;
            Ok(seeds.get("genres").unwrap().to_owned())
        }
    }

    /// Get the list of markets where Spotify is available.
    fn available_markets(&self) -> impl Future<Output = Result<Vec<String>, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("markets")
                .send(token)
                .await?;

            let seeds: HashMap<String, Vec<String>> = pares!(&body)?;
            Ok(seeds.get("markets").unwrap().to_owned())
        }
    }
}
