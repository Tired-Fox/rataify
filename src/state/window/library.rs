use std::ops::{AddAssign, Sub, SubAssign};
use std::{fmt::Debug, ops::Add};
use std::collections::HashMap;
use crossterm::event::KeyEvent;
use strum::EnumCount;

use color_eyre::Result;
use color_eyre::eyre::Error;
use ratatui::widgets::TableState;
use tupy::{api::{flow::{AuthFlow, Pkce}, request::{Query, SearchType, Play}, response::{Paged, FollowedArtists, SavedAudiobooks, SavedAlbums, Paginated, PagedPlaylists, SavedShows}, PublicApi, UserApi, Uri}, Pagination};

use crate::key;
use crate::{state::{IterCollection, Loading, actions::{action_label, Action, GoTo}}, PAGE_SIZE};
use super::Pages;

static USER_PLAYLISTS_FILENAME: &str = "user.playlists.cache";
static USER_FILENAME: &str = "user.id.cache";
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
        let index = (self as usize + rhs) % Self::COUNT;
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
        let index = (self as isize - rhs as isize)  % Self::COUNT as isize;
        if index < 0 {
            Self::from_repr((Self::COUNT as isize + index) as usize).unwrap()
        } else {
            Self::from_repr(index as usize).unwrap()
        }
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
    Results,
}

impl Default for Selection {
    fn default() -> Self {
        Self::SpotifyPlaylist
    }
}

#[derive(Debug, Clone)]
pub struct LibraryState {
    pub user_id: String,
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
    pub async fn tab(&mut self) -> Result<()> {
        self.selected_tab += 1;
        self.result_state.select(Some(0));
        match self.selected_tab {
            LibraryTab::Playlists if self.playlists.items.lock().unwrap().is_none() && self.playlists.pager.lock().await.has_next() => {
                self.playlists.next().await?;
            },
            LibraryTab::Artists if self.artists.items.lock().unwrap().is_none() && self.artists.pager.lock().await.has_next() => {
                self.artists.next().await?;
            },
            LibraryTab::Albums if self.albums.items.lock().unwrap().is_none() && self.albums.pager.lock().await.has_next() => {
                self.albums.next().await?;
            },
            LibraryTab::Shows if self.shows.items.lock().unwrap().is_none() && self.shows.pager.lock().await.has_next() => {
                self.shows.next().await?;
            },
            LibraryTab::Audiobooks if self.audiobooks.items.lock().unwrap().is_none() && self.audiobooks.pager.lock().await.has_next() => {
                self.audiobooks.next().await?;
            }
            _ =>{}
        }
        Ok(())
    }

    pub async fn backtab(&mut self) -> Result<()> {
        self.selected_tab -= 1;
        self.result_state.select(Some(0));
        match self.selected_tab {
            LibraryTab::Playlists if self.playlists.items.lock().unwrap().is_none() && self.playlists.pager.lock().await.has_next() => {
                self.playlists.next().await?;
            },
            LibraryTab::Artists if self.artists.items.lock().unwrap().is_none() && self.artists.pager.lock().await.has_next() => {
                self.artists.next().await?;
            },
            LibraryTab::Albums if self.albums.items.lock().unwrap().is_none() && self.albums.pager.lock().await.has_next() => {
                self.albums.next().await?;
            },
            LibraryTab::Shows if self.shows.items.lock().unwrap().is_none() && self.shows.pager.lock().await.has_next() => {
                self.shows.next().await?;
            },
            LibraryTab::Audiobooks if self.audiobooks.items.lock().unwrap().is_none() && self.audiobooks.pager.lock().await.has_next() => {
                self.audiobooks.next().await?;
            }
            _ =>{}
        }
        Ok(())
    }

    pub async fn right(&mut self) -> Result<()> {
        match self.selection {
            Selection::SpotifyPlaylist => {
                self.selected_spotify_playlist += 1;
            },
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

    pub async fn refresh(&mut self) -> Result<()> {
        if let Selection::Results = self.selection {
            match self.selected_tab {
                LibraryTab::Playlists if self.playlists.items.lock().unwrap().is_some() => {
                    self.playlists.refresh().await?;
                    self.result_state.select(None);
                },
                LibraryTab::Artists if self.artists.items.lock().unwrap().is_some() => {
                    self.artists.refresh().await?;
                    self.result_state.select(None);
                },
                LibraryTab::Albums if self.albums.items.lock().unwrap().is_some() => {
                    self.albums.refresh().await?;
                    self.result_state.select(None);
                },
                LibraryTab::Shows if self.shows.items.lock().unwrap().is_some() => {
                    self.shows.refresh().await?;
                    self.result_state.select(None);
                },
                LibraryTab::Audiobooks if self.audiobooks.items.lock().unwrap().is_some() => {
                    self.audiobooks.refresh().await?;
                    self.result_state.select(None);
                }
                _ =>{}
            }
        }
        Ok(())
    }

    pub async fn down(&mut self) {
        match self.selection {
            Selection::SpotifyPlaylist => {
                let len = match self.selected_tab {
                    LibraryTab::Playlists => if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Artists => if let Some(Loading::Some(items)) = self.artists.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Albums => if let Some(Loading::Some(items)) = self.albums.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Shows => if let Some(Loading::Some(items)) = self.shows.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                    LibraryTab::Audiobooks => if let Some(Loading::Some(items)) = self.audiobooks.items.lock().unwrap().as_ref() {
                        items.items().len()
                    } else { 0 },
                };
                if len != 0 {
                    self.selection = Selection::Results;
                    self.result_state.select(Some(0));
                }
            },
            Selection::Results => {
                match self.selected_tab {
                    LibraryTab::Playlists => if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Artists => if let Some(Loading::Some(items)) = self.artists.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Albums => if let Some(Loading::Some(items)) = self.albums.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Shows => if let Some(Loading::Some(items)) = self.shows.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    },
                    LibraryTab::Audiobooks => if let Some(Loading::Some(items)) = self.audiobooks.items.lock().unwrap().as_ref() {
                        self.result_state.next_in_list(items.items().len());
                    }
                }
            }
        }
    }

    pub async fn up(&mut self) {
        match self.selection {
            Selection::SpotifyPlaylist => {},
            Selection::Results => {
                if self.result_state.selected().unwrap_or(0) == 0 {
                    self.selection = Selection::SpotifyPlaylist;
                    self.result_state.select(None);
                } else {
                    match self.selected_tab {
                        LibraryTab::Playlists => if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Artists => if let Some(Loading::Some(items)) = self.artists.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Albums => if let Some(Loading::Some(items)) = self.albums.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Shows => if let Some(Loading::Some(items)) = self.shows.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        },
                        LibraryTab::Audiobooks => if let Some(Loading::Some(items)) = self.audiobooks.items.lock().unwrap().as_ref() {
                            self.result_state.prev_in_list(items.items().len());
                        }
                    }
                }

            }
        }
    }

    pub fn select(&self) -> Option<Vec<(KeyEvent, Action, &'static str)>> {
        match self.selection {
            Selection::SpotifyPlaylist => match self.selected_spotify_playlist {
                FromSpotify::ReleaseRadar => if let Some(release) = self.user_playlists.release.as_ref() {
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::playlist(release.clone(), None, 0)), action_label::PLAY),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Playlist(release.clone())), action_label::GO_TO_PLAYLIST),
                    ])
                },
                FromSpotify::DiscoverWeekly => if let Some(discover) = self.user_playlists.discover.as_ref() {
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::playlist(discover.clone(), None, 0)), action_label::PLAY),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Playlist(discover.clone())), action_label::GO_TO_PLAYLIST),
                    ])
                },
                FromSpotify::LikedSongs => {
                    let uri = Uri::collection(self.user_id.clone());
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::collection(uri.id(), None, 0)), action_label::PLAY),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::LikedSongs), "Go to Liked Songs"),
                    ])
                },
                FromSpotify::MyEpisodes => {
                    return Some(vec![
                        (key!('C' + SHIFT), Action::GoTo(GoTo::MyEpisodes), "Go to My Episodes"),
                    ])
                },
            },
            Selection::Results => match self.selected_tab {
                LibraryTab::Playlists => if let Some(Loading::Some(items)) = self.playlists.items.lock().unwrap().as_ref() {
                    let item = items.items.get(self.result_state.selected().unwrap_or(0))?;
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::playlist(item.uri.clone(), None, 0)), action_label::PLAY),
                        (key!('R' + SHIFT), Action::remove(item.uri.clone(), |saved| Ok(())), action_label::REMOVE),
                        // TODO: Action to add entire playlist to queue
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Playlist(item.uri.clone())), action_label::GO_TO_PLAYLIST),
                    ])
                },
                LibraryTab::Artists => if let Some(Loading::Some(items)) = self.artists.items.lock().unwrap().as_ref() {
                    let item = items.items.get(self.result_state.selected().unwrap_or(0))?;
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::artist(item.uri.clone())), action_label::PLAY),
                        (key!('R' + SHIFT), Action::remove(item.uri.clone(), |saved| Ok(())), action_label::REMOVE),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Artist(item.uri.clone())), action_label::GO_TO_ARTIST),
                    ])
                },
                LibraryTab::Albums => if let Some(Loading::Some(items)) = self.albums.items.lock().unwrap().as_ref() {
                    let item = items.items.get(self.result_state.selected().unwrap_or(0))?;
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::album(item.album.uri.clone(), None, 0)), action_label::PLAY),
                        (key!('R' + SHIFT), Action::remove(item.album.uri.clone(), |saved| Ok(())), action_label::REMOVE),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Album(item.album.uri.clone())), action_label::GO_TO_ALBUM),
                        if item.album.artists.len() > 1 {
                            (key!('A' + SHIFT), Action::GoTo(GoTo::Artists(item.album.artists.iter().map(|a| (a.uri.clone(), a.name.clone())).collect::<Vec<_>>())), action_label::SELECT_ARTIST)
                        } else {
                            (key!('A' + SHIFT), Action::GoTo(GoTo::Artist(item.album.artists[0].uri.clone())), action_label::GO_TO_ARTIST)
                        }
                    ])
                },
                LibraryTab::Shows => if let Some(Loading::Some(items)) = self.shows.items.lock().unwrap().as_ref() {
                    let item = items.items.get(self.result_state.selected().unwrap_or(0))?;
                    return Some(vec![
                        (key!(Enter), Action::PlayContext(Play::show(item.show.uri.clone(), None, 0)), action_label::PLAY),
                        (key!('R' + SHIFT), Action::remove(item.show.uri.clone(), |saved| Ok(())), action_label::REMOVE),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Show(item.show.uri.clone())), action_label::GO_TO_SHOW),
                    ])
                },
                LibraryTab::Audiobooks => if let Some(Loading::Some(items)) = self.audiobooks.items.lock().unwrap().as_ref() {
                    let item = items.items.get(self.result_state.selected().unwrap_or(0))?;
                    return Some(vec![
                        // TODO: Double check this action
                        (key!(Enter), Action::PlayContext(Play::show(item.uri.clone(), None, 0)), action_label::PLAY),
                        (key!('R' + SHIFT), Action::remove(item.uri.clone(), |saved| Ok(())), action_label::REMOVE),
                        (key!('C' + SHIFT), Action::GoTo(GoTo::Audiobook(item.uri.clone())), action_label::GO_TO_AUDIOBOOK)
                    ])
                }
            }
        }
        None
    }

    pub async fn new(dir: &str, api: &Pkce) -> Result<Self> {
        let cache_playlist_path = dirs::cache_dir().unwrap().join(dir).join(USER_PLAYLISTS_FILENAME);
        let cache_user_id_path = dirs::cache_dir().unwrap().join(dir).join(USER_FILENAME);
        let mut user_playlists = UserPlaylists::default();

        if !cache_playlist_path.exists() {
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
                            user_playlists.release = Some(playlist.uri);
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
                            user_playlists.discover = Some(playlist.uri);
                            break;
                        }
                    }
                }
            }

            std::fs::write(cache_playlist_path, user_playlists.to_cache_string())?;
        } else {
            let cache = std::fs::read_to_string(cache_playlist_path).unwrap();
            for line in cache.lines() {
                match line.split_once("=>") {
                    Some(("Release Radar", uri)) => user_playlists.release = Some(uri.parse().map_err(|e| Error::msg(e))?),
                    Some(("Discover Weekly", uri)) => user_playlists.discover = Some(uri.parse().map_err(|e| Error::msg(e))?),
                    _ => {}
                }
            }
        }

        let user_id = if !cache_user_id_path.exists() {
            if api.token().is_expired() {
                api.refresh().await?;
            }
            let user_id = api.current_user_profile().await?.id.clone();
            std::fs::write(cache_user_id_path, user_id.clone())?;
            user_id
        } else {
            std::fs::read_to_string(cache_user_id_path)?
        };

        let mut layout_state = Self {
            user_id,
            user_playlists,
            selection: Selection::default(),
            selected_spotify_playlist: FromSpotify::default(),
            selected_tab: LibraryTab::default(),
            result_state: TableState::default(),

            playlists: Pages::new(api.playlists::<PAGE_SIZE, _>(None)?),
            artists: Pages::new(api.followed_artists::<PAGE_SIZE>()?),
            albums: Pages::new(api.saved_albums::<PAGE_SIZE, _>(None)?),
            audiobooks: Pages::new(api.saved_audiobooks::<PAGE_SIZE>()?),
            shows: Pages::new(api.saved_shows::<PAGE_SIZE>()?),
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
