use std::{collections::HashMap, future::Future};

use crate::{pares, Error};

use super::{
    flow::AuthFlow,
    request::{self, IncludeGroup, IntoSpotifyId, Query, RecommendationSeed, SearchType},
    response::{
        Album, AlbumTracks, Artist, ArtistAlbums, AudioAnalysis, AudioFeatures, Audiobook, Categories, Category, Chapter, Chapters, Episode, FeaturedPlaylists, Image, NewReleases, PagedPlaylists, Paginated, Playlist, Recommendations, Search, Show, ShowEpisodes, Track
    },
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
    ///
    /// # Arguments
    /// <N> Is the number of items to return per page.
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
            |c: HashMap<String, NewReleases>| c.get("albums").unwrap().to_owned(),
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
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the album.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page.
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
            |c: AlbumTracks| c,
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
    fn artists<D: IntoSpotifyId, I: IntoIterator<Item = D>>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Vec<Artist>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("artists")
                .param(
                    "ids",
                    ids.into_iter()
                        .map(|id| id.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(","),
                )
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
    /// <N> Is the number of items to return per page.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn artist_albums<const N: usize, I, M>(
        &self,
        id: I,
        market: M,
        include_groups: &[IncludeGroup],
    ) -> Result<Paginated<ArtistAlbums, ArtistAlbums, Self, N>, Error>
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
            url.push_str(&format!(
                "&include_groups={}",
                include_groups
                    .iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: ArtistAlbums| c, 
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
    fn artist_top_tracks<I, M>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Vec<Track>, Error>>
    where
        I: IntoSpotifyId,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } =
                request::get!("artists/{}/top-tracks", id.into_spotify_id())
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
    fn related_artists<I: IntoSpotifyId>(
        &self,
        id: I,
    ) -> impl Future<Output = Result<Vec<Artist>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } =
                request::get!("artists/{}/related-artists", id.into_spotify_id())
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
    fn audiobook<I: IntoSpotifyId, M: IntoSpotifyParam>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Audiobook, Error>> {
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
    fn audiobooks<D: IntoSpotifyId, I: IntoIterator<Item = D>, M: IntoSpotifyParam>(
        &self,
        ids: I,
        market: M,
    ) -> impl Future<Output = Result<Vec<Audiobook>, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("audiobooks")
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

            let audiobooks: HashMap<String, Vec<Audiobook>> = pares!(&body)?;
            Ok(audiobooks.get("audiobooks").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information about an audiobook's chapters. Audiobooks are only available within the US, UK, Canada, Ireland, New Zealand and Australia markets.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the audiobook.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn audiobook_chapters<const N: usize, I: IntoSpotifyId, M: IntoSpotifyParam>(
        &self,
        id: I,
        market: M,
    ) -> Result<Paginated<Chapters, Chapters, Self, N>, Error> {
        let mut url = format!(
            "{API_BASE_URL}/audiobooks/{}/chapters?limit={N}",
            id.into_spotify_id()
        );

        if let Some(m) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", m));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: Chapters| c,
        ))
    }

    /// Get a list of categories used to tag items in Spotify (on, for example, the Spotify player’s “Browse” tab).
    ///
    /// # Arguments
    /// - `locale`: The desired language, consisting of a lowercase ISO 639-1 language code and an uppercase ISO 3166-1 alpha-2 country code, joined by an underscore.
    /// <N> Is the number of items to return per page.
    fn browse_categories<const N: usize, L: IntoSpotifyParam>(
        &self,
        locale: L,
    ) -> Result<Paginated<Categories, HashMap<String, Categories>, Self, N>, Error> {
        let mut url = format!("{API_BASE_URL}/browse/categories?limit={N}");

        if let Some(locale) = locale.into_spotify_param() {
            url.push_str(&format!("&locale={}", locale));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: HashMap<String, Categories>| c.get("categories").unwrap().to_owned(),
        ))
    }

    /// Get a single category used to tag items in Spotify (on, for example, the Spotify player’s “Browse” tab).
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the category.
    /// - `locale`: The desired language, consisting of a lowercase ISO 639-1 language code and an uppercase ISO 3166-1 alpha-2 country code, joined by an underscore.
    fn browse_category<I: IntoSpotifyId, L: IntoSpotifyParam>(
        &self,
        id: I,
        locale: L,
    ) -> impl Future<Output = Result<Category, Error>> {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } =
                request::get!("browse/categories/{}", id.into_spotify_id())
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
    fn chapter<I: IntoSpotifyId, M: IntoSpotifyParam>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Chapter, Error>> {
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
    fn chapters<D, I, M>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Vec<Chapter>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("chapters")
                .param(
                    "ids",
                    id.into_iter()
                        .map(|id| id.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(","),
                )
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
    fn episode<I: IntoSpotifyId, M: IntoSpotifyParam>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Episode, Error>> {
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
    fn episodes<D, I, M>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Vec<Episode>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("episodes")
                .param(
                    "ids",
                    id.into_iter()
                        .map(|id| id.into_spotify_id())
                        .collect::<Vec<_>>()
                        .join(","),
                )
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
            let SpotifyResponse { body, .. } =
                request::get!("recommendations/available-genre-seeds")
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
            let SpotifyResponse { body, .. } = request::get!("markets").send(token).await?;

            let seeds: HashMap<String, Vec<String>> = pares!(&body)?;
            Ok(seeds.get("markets").unwrap().to_owned())
        }
    }

    fn search<const N: usize, M: IntoSpotifyParam>(
        &self,
        query: &[Query],
        types: &[SearchType],
        market: M,
        include_external: bool,
    ) -> Result<Search<N, Self>, Error> {
        if query.is_empty() {
            return Err(Error::InvalidArgument("query", "Query must contain at least one search term".to_string()))
        }

        if types.is_empty() {
            return Err(Error::InvalidArgument("types", "Types must contain at least one result type to search for".to_string()))
        }

        let mut url = format!("{API_BASE_URL}/search?{}",
            serde_urlencoded::to_string([
                ("limit", N.to_string()),
                ("q", query.iter().map(|q| q.to_string()).collect::<Vec<_>>().join(" ")),
            ])?
        );

        if let Some(m) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", m));
        }

        if include_external {
            url.push_str("&include_external=audio");
        }

        Ok(Search::new(self.clone(), url.as_str(), types))
    }

    /// Get Spotify catalog information for a single show identified by its unique Spotify ID.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the show.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Scopes
    /// - `user-read-playback-position` [Optional]: Read your position in content you have played.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn show<I: IntoSpotifyId, M: IntoSpotifyParam>(&self, id: I, market: M) -> impl Future<Output = Result<Show, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("shows/{}", id.into_spotify_id())
                .param("market", market)
                .send(token)
                .await?;
            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for several shows based on their Spotify IDs.
    ///
    /// # Arguments
    /// - `ids`: A list of the [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the shows. Maximum: 50 IDs.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn shows<D, I, M>(&self, ids: I, market: M) -> impl Future<Output = Result<Vec<Show>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("shows")
                .param("ids", ids.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .param("market", market)
                .send(token)
                .await?;

            let shows: HashMap<String, Vec<Show>> = pares!(&body)?;
            Ok(shows.get("shows").unwrap().to_owned())
        }
    }

    /// Get Spotify catalog information about an show’s episodes. Optional parameters can be used to limit the number of episodes returned.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the show.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page.
    ///
    /// # Scopes
    /// - `user-read-playback-position` [Optional]: Read your position in content you have played.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn show_episodes<const N: usize, I, M>(
        &self,
        id: I,
        market: M,
    ) -> Result<Paginated<ShowEpisodes, ShowEpisodes, Self, N>, Error>
    where
        I: IntoSpotifyId,
        M: IntoSpotifyParam,
    {
        let mut url = format!(
            "{API_BASE_URL}/shows/{}/episodes?limit={N}",
            id.into_spotify_id(),
        );

        if let Some(m) = market.into_spotify_param() {
            url.push_str(&format!("&market={}", m));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: ShowEpisodes| c,
        ))
    }

    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    /// 
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the track.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn track<I: IntoSpotifyId, M: IntoSpotifyParam>(&self, id: I, market: M) -> impl Future<Output = Result<Track, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("tracks/{}", id.into_spotify_id())
                .param("market", market)
                .send(token)
                .await?;
            Ok(pares!(&body)?)
        }
    }

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    /// 
    /// # Arguments
    /// - `ids`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the tracks. Maximum: 100 IDs.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    fn tracks<D, I, M>(&self, id: I, market: M) -> impl Future<Output = Result<Vec<Track>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
        M: IntoSpotifyParam,
    {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("tracks")
                .param("ids", id.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .param("market", market)
                .send(token)
                .await?;

            let tracks: HashMap<String, Vec<Track>> = pares!(&body)?;
            Ok(tracks.get("tracks").unwrap().to_owned())
        }
    }

    /// Get audio feature information for a single track identified by its unique Spotify ID.
    /// 
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the track.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be used to train machine learning or AI models](https://developer.spotify.com/terms#section-iv-restrictions)
    fn track_audio_feature<I: IntoSpotifyId>(&self, id: I) -> impl Future<Output = Result<AudioFeatures, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("audio-features/{}", id.into_spotify_id())
                .send(token)
                .await?;
            Ok(pares!(&body)?)
        }
    }

    /// Get audio features for multiple tracks based on their Spotify IDs.
    ///
    /// # Arguments
    /// - `ids`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the tracks. Maximum: 100 IDs.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be used to train machine learning or AI models](https://developer.spotify.com/terms#section-iv-restrictions)
    fn track_audio_features<D, I>(&self, ids: I) -> impl Future<Output = Result<Vec<AudioFeatures>, Error>>
    where
        D: IntoSpotifyId,
        I: IntoIterator<Item = D>,
    {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("audio-features")
                .param("ids", ids.into_iter().map(|id| id.into_spotify_id()).collect::<Vec<_>>().join(","))
                .send(token)
                .await?;

            let features: HashMap<String, Vec<AudioFeatures>> = pares!(&body)?;
            Ok(features.get("audio_features").unwrap().to_owned())
        }
    }

    /// Get a low-level audio analysis for a track in the Spotify catalog. The audio analysis describes the track’s structure and musical content, including rhythm, pitch, and timbre.
    /// 
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) for the track.
    fn track_audio_analysis<I: IntoSpotifyId>(&self, id: I) -> impl Future<Output = Result<AudioAnalysis, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("audio-analysis/{}", id.into_spotify_id())
                .send(token)
                .await?;
            Ok(pares!(&body)?)
        }
    }

    /// Recommendations are generated based on the available information for a given seed entity and matched against similar artists and tracks. If there is sufficient information about the provided seeds, a list of tracks will be returned together with pool size details.
    ///
    /// For artists and tracks that are very new or obscure there might not be enough data to generate a list of tracks.
    ///
    /// # Arguments
    /// - `seed`: A collection of optional parameters to customize the result and seed the recommendations engine.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    /// <N> Is the number of items to return per page.
    ///
    /// # Important Policy Notes
    /// - Spoitfy [content may not be used to train machine learning or AI models](https://developer.spotify.com/terms#section-iv-restrictions)
    fn recommendations<const N: usize, M: IntoSpotifyParam>(
        &self,
        market: M,
        seed: RecommendationSeed,
    ) -> impl Future<Output = Result<Recommendations, Error>> {
        async move {
            let token = self.token().await?;

            let mut url = format!("recommendations?limit={N}&{}", seed.into_params()?);
            if let Some(m) = market.into_spotify_param() {
                url.push_str(&format!("&market={}", m));
            }

            let SpotifyResponse { body, .. } = request::get!(url)
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a playlist owned by a Spotify user.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    /// - `market`: An [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2). If a country code is specified, only content that is available in that market will be returned. If a valid user access token is specified in the request header, the country associated with the user account will take priority over this parameter.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    /// - Spotify [content may not be used to train machine learning or AI models](https://developer.spotify.com/terms#section-iv-restrictions)
    fn playlist<I: IntoSpotifyId, M: IntoSpotifyParam>(
        &self,
        id: I,
        market: M,
    ) -> impl Future<Output = Result<Playlist, Error>> {
        // TODO: What do do about the `fields` param? It narrows down what is returned. This could
        // be nice but would be very complex to suppport using concrete types.
        // https://open.spotify.com/playlist/7AXnDxOcbYCymLv2krA3Hx?si=fc0115d894bd481f
        async move {
            let token = self.token().await?;

            let SpotifyResponse { body, .. } = request::get!("playlists/{}", id.into_spotify_id())
                .param("market", market)
                // NOTE: This is manually maintained. T
                .param("additional_types", "track,episode")
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }

    /// Get a list of Spotify featured playlists (shown, for example, on a Spotify player's 'Browse' tab).
    /// 
    /// # Arguments
    /// - `locale`: The desired language, consisting of an [ISO 639-1 language code](http://en.wikipedia.org/wiki/List_of_ISO_639-1_codes) and an [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2), joined by an underscore. For example: `es_MX`, meaning "Spanish (Mexico)". Provide this parameter if you want the list of returned items to be in a particular language (where available).
    /// <N> Is the number of items to return per page. Maximum: 50
    ///
    /// # Important Policy Notes
    /// - Spotify [data may not be transferred](https://developer.spotify.com/policy/#iii-some-prohibited-applications)
    fn featured_playlists<const N: usize, L: IntoSpotifyParam>(&self, locale: L) -> Result<Paginated<PagedPlaylists, FeaturedPlaylists, Self, N>, Error> {
        let mut url = format!("{API_BASE_URL}/browse/featured-playlists?limit={N}");

        if let Some(locale) = locale.into_spotify_param() {
            url.push_str(&format!("&locale={}", locale));
        }

        Ok(Paginated::new(
            self.clone(),
            Some(url),
            None,
            |c: FeaturedPlaylists| c.playlists,
        ))
    }

    /// Get a list of Spotify playlists tagged with a particular category.
    /// 
    /// # Arguments
    /// - `id`: The [Spotify category ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) of the category.
    /// <N> Is the number of items to return per page. Maximum: 50
    fn category_playlists<const N: usize, I: IntoSpotifyId>(&self, id: I) -> Result<Paginated<PagedPlaylists, FeaturedPlaylists, Self, N>, Error> {
        Ok(Paginated::new(
            self.clone(),
            Some(format!("{API_BASE_URL}/browse/categories/{}/playlists?limit={N}", id.into_spotify_id())),
            None,
            |c: FeaturedPlaylists| c.playlists,
        ))
    }

    /// Get the current image associated with a specific playlist.
    ///
    /// # Arguments
    /// - `id`: The [Spotify ID](https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids) of the playlist.
    ///
    /// # Important Policy Notes
    /// - Spotify [content may not be downloaded](https://developer.spotify.com/terms/#section-iv-restrictions)
    /// - Keep visual content in it's [original form](https://developer.spotify.com/documentation/design#using-our-content)
    /// - Ensure content [attribution](https://developer.spotify.com/policy/#ii-respect-content-and-creators)
    /// - Spotify [content may not be used to train machine learning or AI models](https://developer.spotify.com/terms#section-iv-restrictions)
    fn playlist_cover_image<I: IntoSpotifyId>(&self, id: I) -> impl Future<Output = Result<Vec<Image>, Error>> {
        async move {
            let token = self.token().await?;
            let SpotifyResponse { body, .. } = request::get!("playlists/{}/images", id.into_spotify_id())
                .send(token)
                .await?;

            Ok(pares!(&body)?)
        }
    }
}
