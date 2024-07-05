use std::ops::{AddAssign, Sub, SubAssign};
use std::{fmt::Debug, ops::Add};
use std::collections::HashMap;
use strum::EnumCount;
use tokio::sync::Mutex;

use color_eyre::Result;
use color_eyre::eyre::Error;
use ratatui::widgets::TableState;
use serde::Deserialize;
use tupy::{api::{flow::{AuthFlow, Pkce}, request::{Query, SearchType}, response::{Paged, FollowedArtists, SavedAudiobooks, SavedAlbums, Paginated, PagedPlaylists, SavedShows}, PublicApi, UserApi, Uri}, Pagination};

use crate::state::{IterCollection, Loading};
use crate::{Locked, Shared};

static USER_PLAYLISTS_FILENAME: &str = "user.playlists.cache";

#[derive(Debug, Clone, Default)]
pub struct UserPlaylists {
    pub release: Option<Uri>,
    pub discover: Option<Uri>,
}

impl UserPlaylists {
    pub fn to_cache_string(&self) -> String {
        let mut buffer = Vec::new();
        if let Some(release) = &self.release {
            buffer.push(format!("Release Radar=>{release}"));
        }

        if let Some(discover) = &self.discover {
            buffer.push(format!("Discover Weekly=>{discover}"));
        }
        buffer.join("\n")
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, strum_macros::IntoStaticStr, strum_macros::EnumIter, strum_macros::FromRepr, strum_macros::EnumCount)]
pub enum FromSpotify {
    #[default]
    #[strum(serialize = "Release Radar")]
    ReleaseRadar,
    #[strum(serialize = "Discover Weekly")]
    DiscoverWeekly,
    #[strum(serialize = "Liked Songs")]
    LikedSongs,
    #[strum(serialize = "My Episodes")]
    MyEpisodes,
}

impl Add<usize> for FromSpotify {
    type Output = Self;
    
    fn add(self, rhs: usize) -> Self::Output {
        let index = (self as usize + rhs).min(Self::COUNT - 1);
        Self::from_repr(index).unwrap()
    }
}

impl AddAssign<usize> for FromSpotify {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for FromSpotify {
    type Output = Self;
    
    fn sub(self, rhs: usize) -> Self::Output {
        let index = (self as usize).saturating_sub(rhs);
        Self::from_repr(index).unwrap()
    }
}

impl SubAssign<usize> for FromSpotify {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl FromSpotify {
    #[inline]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::ReleaseRadar => "Release Radar",
            Self::DiscoverWeekly => "Discover Weekly",
            Self::LikedSongs => "Liked Songs",
            Self::MyEpisodes => "My Episodes",
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, strum_macros::IntoStaticStr, strum_macros::EnumIter, strum_macros::FromRepr, strum_macros::EnumCount)]
pub enum LibraryTab {
    #[default]
    Playlists,
    Artists,
    Albums,
    Shows,
    Audiobooks,
}

impl Add<usize> for LibraryTab {
    type Output = Self;
    
    fn add(self, rhs: usize) -> Self::Output {
        let index = (self as usize + rhs).min(Self::COUNT - 1);
        Self::from_repr(index).unwrap()
    }
}

impl AddAssign<usize> for LibraryTab {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for LibraryTab {
    type Output = Self;
    
    fn sub(self, rhs: usize) -> Self::Output {
        let index = (self as usize).saturating_sub(rhs);
        Self::from_repr(index).unwrap()
    }
}

impl SubAssign<usize> for LibraryTab {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl LibraryTab {
    #[inline]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Playlists => "Playlists",
            Self::Artists => "Artists",
            Self::Albums => "Albums",
            Self::Shows => "Shows",
            Self::Audiobooks => "Audiobooks",
        }
    }
}

#[derive(Debug, Clone, PartialEq, strum_macros::EnumIs)]
pub enum Selection {
    SpotifyPlaylist,
    Tabs,
    Results,
}

impl Default for Selection {
    fn default() -> Self {
        Self::SpotifyPlaylist
    }
}

#[derive(Debug, Clone)]
pub struct Pages<R, P>
    where 
        R: Clone + Debug + Send,
        P: Clone + Debug + Send,
{
    pub pager: Shared<Mutex<Paginated<R, P, Pkce, 20>>>,
    pub items: Shared<Locked<Loading<R>>>,
}

impl<R, P> Pages<R, P>
    where 
        R: Clone + Debug + Send + Paged + 'static,
        P: Clone + Debug + Send + Deserialize<'static> + 'static,
{
    pub fn new(pager: Paginated<R, P, Pkce, 20>) -> Self {
        Self {
            pager: Shared::new(Mutex::new(pager)),
            items: Shared::default(),
        }
    }

    pub async fn next(&mut self) -> Result<()> {
        *self.items.lock().unwrap() = Loading::Loading;

        let items = self.items.clone();
        let pager = self.pager.clone();
        tokio::task::spawn(async move {
            let next = pager.lock().await.next().await.unwrap();
            *items.lock().unwrap() = Loading::from(next);
        });
        Ok(())
    }

    pub async fn prev(&mut self) -> Result<()> {
        *self.items.lock().unwrap() = Loading::Loading;

        let items = self.items.clone();
        let pager = self.pager.clone();
        tokio::spawn(async move {
            *items.lock().unwrap() = Loading::from(pager.lock().await.prev().await.unwrap());
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LibraryState {
    pub user_playlists: UserPlaylists,
    pub selection: Selection,

    pub selected_spotify_playlist: FromSpotify,
    pub selected_tab: LibraryTab,
    pub result_state: TableState,

    // State for paginated results
    pub playlists: Pages<PagedPlaylists, PagedPlaylists>,
    pub artists: Pages<FollowedArtists, HashMap<String, FollowedArtists>>,
    pub albums: Pages<SavedAlbums, SavedAlbums>,
    pub shows: Pages<SavedShows, SavedShows>,
    pub audiobooks: Pages<SavedAudiobooks, SavedAudiobooks>,
}

impl LibraryState {
    pub async fn tab(&mut self) {
        match self.selection {
            Selection::SpotifyPlaylist => {
                if self.selected_spotify_playlist as usize >= FromSpotify::COUNT - 1 {
                    let len = match self.selected_tab {
                        LibraryTab::Playlists => if let Loading::Some(items) = self.playlists.items.lock().unwrap().as_ref() {
                            items.items().len()
                        } else { 0 },
                        LibraryTab::Artists => if let Loading::Some(items) = self.artists.items.lock().unwrap().as_ref() {
                            items.items().len()
                        } else { 0 },
                        LibraryTab::Albums => if let Loading::Some(items) = self.albums.items.lock().unwrap().as_ref() {
                            items.items().len()
                        } else { 0 },
                        LibraryTab::Shows => if let Loading::Some(items) = self.shows.items.lock().unwrap().as_ref() {
                            items.items().len()
                        } else { 0 },
                        LibraryTab::Audiobooks => if let Loading::Some(items) = self.audiobooks.items.lock().unwrap().as_ref() {
                            items.items().len()
                        } else { 0 },
                    };
                    if len != 0 {
                        self.selection = Selection::Results;
                        self.result_state.select(Some(0));
                    }
                } else {
                    self.selected_spotify_playlist += 1;
                }
            },
            Selection::Tabs => {
                self.selection = Selection::Results;
                self.result_state.select(Some(0));
            },
            Selection::Results => {
                match self.selected_tab {
                    LibraryTab::Playlists => if let Loading::Some(items) = self.playlists.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Artists => if let Loading::Some(items) = self.artists.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Albums => if let Loading::Some(items) = self.albums.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Shows => if let Loading::Some(items) = self.shows.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Audiobooks => if let Loading::Some(items) = self.audiobooks.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    }
                }
            }
        }
    }

    pub async fn backtab(&mut self) {
        match self.selection {
            Selection::SpotifyPlaylist => {
                self.selected_spotify_playlist -= 1;
            },
            Selection::Tabs => {
                self.selection = Selection::SpotifyPlaylist;
                self.result_state.select(None);
            },
            Selection::Results => {
                if self.result_state.selected().unwrap_or(0) == 0 {
                    self.selection = Selection::SpotifyPlaylist;
                    self.result_state.select(None);
                } else {
                    match self.selected_tab {
                        LibraryTab::Playlists => if let Loading::Some(items) = self.playlists.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Artists => if let Loading::Some(items) = self.artists.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Albums => if let Loading::Some(items) = self.albums.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Shows => if let Loading::Some(items) = self.shows.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Audiobooks => if let Loading::Some(items) = self.audiobooks.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                    }
                }
            }
        }
    }

    pub async fn right(&mut self) -> Result<()> {
        match self.selection {
            Selection::SpotifyPlaylist => {
                self.selected_spotify_playlist += 1;
            },
            Selection::Tabs => {
                self.selected_tab += 1;
            }
            Selection::Results => {
                match self.selected_tab {
                    LibraryTab::Playlists if self.playlists.items.lock().unwrap().is_some() && self.playlists.pager.lock().await.has_next() => {
                        self.playlists.next().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Artists if self.artists.items.lock().unwrap().is_some() && self.artists.pager.lock().await.has_next() => {
                        self.artists.next().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Albums if self.albums.items.lock().unwrap().is_some() && self.albums.pager.lock().await.has_next() => {
                        self.albums.next().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Shows if self.shows.items.lock().unwrap().is_some() && self.shows.pager.lock().await.has_next() => {
                        self.shows.next().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Audiobooks if self.audiobooks.items.lock().unwrap().is_some() && self.audiobooks.pager.lock().await.has_next() => {
                        self.audiobooks.next().await?;
                        self.result_state.select(Some(0));
                    }
                    _ =>{}
                }
            }
        }
        Ok(())
    }

    pub async fn left(&mut self) -> Result<()> {
        match self.selection {
            Selection::SpotifyPlaylist => {
                self.selected_spotify_playlist -= 1;
            },
            Selection::Tabs => {
                self.selected_tab -= 1;
            }
            Selection::Results => {
                match self.selected_tab {
                    LibraryTab::Playlists if self.playlists.items.lock().unwrap().is_some() && self.playlists.pager.lock().await.has_prev() => {
                        self.playlists.prev().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Artists if self.artists.items.lock().unwrap().is_some() && self.artists.pager.lock().await.has_prev() => {
                        self.artists.prev().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Albums if self.albums.items.lock().unwrap().is_some() && self.albums.pager.lock().await.has_prev() => {
                        self.albums.prev().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Shows if self.shows.items.lock().unwrap().is_some() && self.shows.pager.lock().await.has_prev() => {
                        self.shows.prev().await?;
                        self.result_state.select(Some(0));
                    },
                    LibraryTab::Audiobooks if self.audiobooks.items.lock().unwrap().is_some() && self.audiobooks.pager.lock().await.has_prev() => {
                        self.audiobooks.prev().await?;
                        self.result_state.select(Some(0));
                    }
                    _ =>{}
                }
            }
        }
        Ok(())
    }

    pub async fn down(&mut self) {
        match self.selection {
            Selection::SpotifyPlaylist => {
                self.selection = Selection::Tabs;
            },
            Selection::Tabs => {
                let len = match self.selected_tab {
                    LibraryTab::Playlists => if let Loading::Some(items) = self.playlists.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Artists => if let Loading::Some(items) = self.artists.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Albums => if let Loading::Some(items) = self.albums.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Shows => if let Loading::Some(items) = self.shows.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Audiobooks => if let Loading::Some(items) = self.audiobooks.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                };
                if len != 0 {
                    self.selection = Selection::Results;
                    self.result_state.select(Some(0));
                }
            }
            Selection::Results => {
                match self.selected_tab {
                    LibraryTab::Playlists => if let Loading::Some(items) = self.playlists.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Artists => if let Loading::Some(items) = self.artists.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Albums => if let Loading::Some(items) = self.albums.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Shows => if let Loading::Some(items) = self.shows.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Audiobooks => if let Loading::Some(items) = self.audiobooks.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    }
                }
            }
        }
    }

    pub async fn up(&mut self) {
        match self.selection {
            Selection::SpotifyPlaylist => {},
            Selection::Tabs => {
                self.selection = Selection::SpotifyPlaylist;
            }
            Selection::Results => {
                if self.result_state.selected().unwrap_or(0) == 0 {
                    self.selection = Selection::Tabs;
                    self.result_state.select(None);
                } else {
                    match self.selected_tab {
                        LibraryTab::Playlists => if let Loading::Some(items) = self.playlists.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Artists => if let Loading::Some(items) = self.artists.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Albums => if let Loading::Some(items) = self.albums.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Shows => if let Loading::Some(items) = self.shows.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Audiobooks => if let Loading::Some(items) = self.audiobooks.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        }
                    }
                }

            }
        }
    }

    pub async fn new(dir: &str, api: &Pkce) -> Result<Self> {
        let path = dirs::cache_dir().unwrap().join(dir).join(USER_PLAYLISTS_FILENAME);
        let mut user_playlists = UserPlaylists::default();

        if !path.exists() {
            if api.token().is_expired() {
                api.refresh().await?;
            }


            // Search for Release Radar playlist on spotify. It is done this way as the user may
            // not follow the playlist.
            let mut search = api.search::<2, _>(&[Query::text("Release Radar")], &[SearchType::Playlist], None, false)?;
            if let Some(playlists) = search.playlists() {
                if let Some(page) = playlists.next().await? {
                    for playlist in page.items {
                        if playlist.name.as_str() == "Release Radar" && playlist.owner.id.as_str() == "spotify" {
                            user_playlists.release = Some(playlist.uri.parse().map_err(|e| Error::msg(e))?);
                            break;
                        }
                    }
                }
            }

            // Search for Discover Weekly playlist on spotify. It is done this way as the user may
            // not follow the playlist.
            let mut search = api.search::<2, _>(&[Query::text("Discover Weekly")], &[SearchType::Playlist], None, false)?;
            if let Some(playlists) = search.playlists() {
                if let Some(page) = playlists.next().await? {
                    for playlist in page.items {
                        if playlist.name.as_str() == "Discover Weekly" && playlist.owner.id.as_str() == "spotify" {
                            user_playlists.discover = Some(playlist.uri.parse().map_err(|e| Error::msg(e))?);
                            break;
                        }
                    }
                }
            }

            std::fs::write(path, user_playlists.to_cache_string())?;
        } else {
            let cache = std::fs::read_to_string(path).unwrap();
            for line in cache.lines() {
                match line.split_once("=>") {
                    Some(("Release Radar", uri)) => user_playlists.release = Some(uri.parse().map_err(|e| Error::msg(e))?),
                    Some(("Discover Weekly", uri)) => user_playlists.discover = Some(uri.parse().map_err(|e| Error::msg(e))?),
                    _ => {}
                }
            }
        }

        let mut layout_state = Self {
            user_playlists,
            selection: Selection::default(),
            selected_spotify_playlist: FromSpotify::default(),
            selected_tab: LibraryTab::default(),
            result_state: TableState::default(),

            playlists: Pages::new(api.playlists::<20, _>(None)?),
            artists: Pages::new(api.followed_artists::<20>()?),
            albums: Pages::new(api.saved_albums::<20, _>(None)?),
            audiobooks: Pages::new(api.saved_audiobooks::<20>()?),
            shows: Pages::new(api.saved_shows::<20>()?),
        };
        match layout_state.selected_tab {
            LibraryTab::Playlists => layout_state.playlists.next().await?,
            LibraryTab::Artists => layout_state.artists.next().await?,
            LibraryTab::Albums => layout_state.albums.next().await?,
            LibraryTab::Shows => layout_state.shows.next().await?,
            LibraryTab::Audiobooks => layout_state.audiobooks.next().await?,
        }
        Ok(layout_state)
    }
}
