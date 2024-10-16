use crate::{api::request::SearchType, impl_paged};
use serde::Deserialize;
use std::collections::HashMap;

use super::{
    Artist, Audiobook, AuthFlow, Paged, Paginated, Show, SimplifiedAlbum, SimplifiedEpisode,
    SimplifiedPlaylist, Track,
};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Albums {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SimplifiedAlbum>,
}

impl_paged!(Albums<SimplifiedAlbum>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Tracks {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Track>,
}
impl_paged!(Tracks<Track>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Artists {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Artist>,
}
impl_paged!(Artists<Artist>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Playlists {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SimplifiedPlaylist>,
}
impl_paged!(Playlists<SimplifiedPlaylist>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Shows {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Show>,
}
impl_paged!(Shows<Show>);

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct Episodes {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<SimplifiedEpisode>,
}
impl_paged!(Episodes<SimplifiedEpisode>);

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct Audiobooks {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items.
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items.
    pub previous: Option<String>,
    /// The total number of items available to return.
    pub total: usize,
    pub items: Vec<Audiobook>,
}
impl_paged!(Audiobooks<Audiobook>);

#[derive(Debug, Clone, PartialEq)]
pub struct Search<const N: usize, F: AuthFlow> {
    pub(crate) albums: Option<Paginated<Albums, HashMap<String, Albums>, F, N>>,
    pub(crate) tracks: Option<Paginated<Tracks, HashMap<String, Tracks>, F, N>>,
    pub(crate) artists: Option<Paginated<Artists, HashMap<String, Artists>, F, N>>,
    pub(crate) playlists: Option<Paginated<Playlists, HashMap<String, Playlists>, F, N>>,
    pub(crate) shows: Option<Paginated<Shows, HashMap<String, Shows>, F, N>>,
    pub(crate) episodes: Option<Paginated<Episodes, HashMap<String, Episodes>, F, N>>,
    pub(crate) audiobooks: Option<Paginated<Audiobooks, HashMap<String, Audiobooks>, F, N>>,
}

impl<const N: usize, F: AuthFlow> Search<N, F> {
    pub fn artists(&mut self) -> Option<&mut Paginated<Artists, HashMap<String, Artists>, F, N>> {
        self.artists.as_mut()
    }

    pub fn albums(&mut self) -> Option<&mut Paginated<Albums, HashMap<String, Albums>, F, N>> {
        self.albums.as_mut()
    }

    pub fn tracks(&mut self) -> Option<&mut Paginated<Tracks, HashMap<String, Tracks>, F, N>> {
        self.tracks.as_mut()
    }

    pub fn playlists(
        &mut self,
    ) -> Option<&mut Paginated<Playlists, HashMap<String, Playlists>, F, N>> {
        self.playlists.as_mut()
    }

    pub fn shows(&mut self) -> Option<&mut Paginated<Shows, HashMap<String, Shows>, F, N>> {
        self.shows.as_mut()
    }

    pub fn episodes(
        &mut self,
    ) -> Option<&mut Paginated<Episodes, HashMap<String, Episodes>, F, N>> {
        self.episodes.as_mut()
    }

    pub fn audiobooks(
        &mut self,
    ) -> Option<&mut Paginated<Audiobooks, HashMap<String, Audiobooks>, F, N>> {
        self.audiobooks.as_mut()
    }

    pub fn new(flow: F, url: &str, types: &[SearchType]) -> Self {
        Self {
            albums: types
                .contains(&SearchType::Album)
                .then(|| Self::create_paginated::<Albums>(flow.clone(), url, SearchType::Album)),
            tracks: types
                .contains(&SearchType::Track)
                .then(|| Self::create_paginated::<Tracks>(flow.clone(), url, SearchType::Track)),
            artists: types
                .contains(&SearchType::Artist)
                .then(|| Self::create_paginated::<Artists>(flow.clone(), url, SearchType::Artist)),
            playlists: types.contains(&SearchType::Playlist).then(|| {
                Self::create_paginated::<Playlists>(flow.clone(), url, SearchType::Playlist)
            }),
            shows: types
                .contains(&SearchType::Show)
                .then(|| Self::create_paginated::<Shows>(flow.clone(), url, SearchType::Show)),
            episodes: types.contains(&SearchType::Episode).then(|| {
                Self::create_paginated::<Episodes>(flow.clone(), url, SearchType::Episode)
            }),
            audiobooks: types.contains(&SearchType::Audiobook).then(|| {
                Self::create_paginated::<Audiobooks>(flow.clone(), url, SearchType::Audiobook)
            }),
        }
    }

    fn create_paginated<P: Paged + Clone>(
        flow: F,
        url: &str,
        t: SearchType,
    ) -> Paginated<P, HashMap<String, P>, F, N> {
        let typ = format!("{t}s");
        Paginated::new(
            flow,
            Some(format!("{url}&type={t}")),
            None,
            move |c: HashMap<String, P>| c.get(typ.as_str()).unwrap().to_owned(),
        )
    }
}
