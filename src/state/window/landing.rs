use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, widgets::TableState};
use ratatui_image::{picker::Picker, protocol::Protocol, Resize};
use strum::EnumCount;
use tupy::api::{flow::Pkce, request::{IncludeGroup, Play}, response::{Album, AlbumTracks, Artist, ArtistAlbums, Audiobook, Chapters, Item, Playlist, PlaylistItemInfo, PlaylistItems, Show, ShowEpisodes, Track}, PublicApi, Uri};

use super::Pages;
use crate::{key, state::{IterCollection, Loading}, ui::{Action, ActionLabel, IntoActions}, Locked, Shared};

#[derive(Default, Debug, Clone, Copy)]
pub enum ArtistLanding {
    #[default]
    Tracks,
    Albums,
}

pub struct Cover {
    pub picker: Picker,
    pub image: Box<dyn Protocol>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, strum_macros::EnumIter, strum_macros::FromRepr, strum_macros::EnumCount)]
pub enum LandingSection {
    Context,
    #[default]
    Content,
}

#[derive(Default)]
pub enum Landing {
    #[default]
    None,
    Artist {
        cover: Shared<Locked<Loading<Cover>>>,
        artist: Artist,
        top_tracks: Vec<Track>,
        albums: Pages<ArtistAlbums, ArtistAlbums>,

        section: ArtistLanding,
        state: TableState,
        landing_section: LandingSection,
    },
    Playlist {
        cover: Shared<Locked<Loading<Cover>>>,
        playlist: Playlist,
        pages: Pages<PlaylistItems, PlaylistItems>,
        state: TableState,
        section: LandingSection,
    },
    Album {
        cover: Shared<Locked<Loading<Cover>>>,
        album: Album,
        pages: Pages<AlbumTracks, AlbumTracks>,
        state: TableState,
        section: LandingSection,
    },
    Show {
        cover: Shared<Locked<Loading<Cover>>>,
        show: Show,
        pages: Pages<ShowEpisodes, ShowEpisodes>,
        state: TableState,
        section: LandingSection,
    },
    Audiobook{
        cover: Shared<Locked<Loading<Cover>>>,
        audiobook: Audiobook,
        pages: Pages<Chapters, Chapters>,
        state: TableState,
        section: LandingSection,
    },
}

impl std::fmt::Debug for Landing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Landing::None => write!(f, "None"),
            Landing::Playlist{..} => write!(f, "Playlist"),
            Landing::Album{..} => write!(f, "Album"),
            Landing::Show{..} => write!(f, "Show"),
            Landing::Audiobook{..} => write!(f, "Audiobook"),
            Landing::Artist{..} => write!(f, "Artist"),
        }
    }
}

async fn get_cover(image: String) -> Option<Cover> {
    image::load_from_memory_with_format(
        reqwest::Client::new()
            .get(image)
            .send().await.ok()?
            .bytes().await.ok()?
            .to_vec()
            .as_slice(),
        image::ImageFormat::Jpeg
    ).ok().as_ref().map(|i| {
        let mut picker = Picker::new((7, 16));
        Cover {
            image: picker.new_protocol(i.clone(), Rect::new(0, 0, 30, 16), Resize::Fit(None)).unwrap(),
            picker,
        }
    })
}

impl Landing {
    pub async fn playlist(api: &Pkce, playlist: Uri) -> Result<Self> {
        let pages = Pages::new(api.playlist_items(playlist.id(), None)?);

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.unwrap();
        });

        let playlist = api.playlist(playlist.id(), None).await?;
        let cover = match playlist.images.as_ref() {
            None => Shared::new(Locked::new(Loading::None)),
            Some(images) => match images.first().as_ref() {
                None => Shared::new(Locked::new(Loading::None)),
                Some(i) => {
                    let cover: Shared<Locked<Loading<Cover>>> = Shared::default();

                    let c = cover.clone();
                    let url = i.url.clone();
                    tokio::spawn(async move {
                        *c.lock().unwrap() = get_cover(url).await.into();
                    });

                    cover
                },
            },
        };

        Ok(Self::Playlist {
            cover,
            playlist,
            pages,
            state: TableState::default(),
            section: LandingSection::default(),
        })
    }

    pub async fn album(api: &Pkce, album: Uri) -> Result<Self> {
        let pages = Pages::new(api.album_tracks(album.id(), None)?);

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.unwrap();
        });

        let album = api.album(album.id(), None).await?;
        let cover = match album.images.first().as_ref() {
            None => Shared::new(Locked::new(Loading::None)),
            Some(i) => {
                let cover: Shared<Locked<Loading<Cover>>> = Shared::default();

                let c = cover.clone();
                let url = i.url.clone();
                tokio::spawn(async move {
                    *c.lock().unwrap() = get_cover(url).await.into();
                });

                cover
            },
        };

        Ok(Self::Album {
            cover,
            album,
            pages,
            state: TableState::default(),
            section: LandingSection::default()
        })
    }

    pub async fn show(api: &Pkce, show: Uri) -> Result<Self> {
        let pages = Pages::new(api.show_episodes(show.id(), None)?);

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.unwrap();
        });

        let show = api.show(show.id(), None).await?;
        let cover = match show.images.first().as_ref() {
            None => Shared::new(Locked::new(Loading::None)),
            Some(i) => {
                let cover: Shared<Locked<Loading<Cover>>> = Shared::default();

                let c = cover.clone();
                let url = i.url.clone();
                tokio::spawn(async move {
                    *c.lock().unwrap() = get_cover(url).await.into();
                });

                cover
            },
        };

        Ok(Self::Show {
            cover,
            show,
            pages,
            state: TableState::default(),
            section: LandingSection::default()
        })
    }

    pub async fn audiobook(api: &Pkce, audiobook: Uri) -> Result<Self> {
        let pages = Pages::new(api.audiobook_chapters(audiobook.id(), None)?);

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.unwrap();
        });

        let audiobook = api.audiobook(audiobook.id(), None).await?;
        let cover = match audiobook.images.first().as_ref() {
            None => Shared::new(Locked::new(Loading::None)),
            Some(i) => {
                let cover: Shared<Locked<Loading<Cover>>> = Shared::default();

                let c = cover.clone();
                let url = i.url.clone();
                tokio::spawn(async move {
                    *c.lock().unwrap() = get_cover(url).await.into();
                });

                cover
            },
        };

        Ok(Self::Audiobook {
            cover,
            audiobook,
            pages,
            state: TableState::default(),
            section: LandingSection::default()
        })
    }

    pub async fn artist(api: &Pkce, uri: Uri) -> Result<Self> {
        let pages = Pages::new(api.artist_albums(uri.id(), None, &[IncludeGroup::Single, IncludeGroup::Album, IncludeGroup::AppearsOn])?);

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.unwrap();
        });

        let artist = api.artist(uri.id()).await?;
        let cover = match artist.images.first().as_ref() {
            None => Shared::new(Locked::new(Loading::None)),
            Some(i) => {
                let cover: Shared<Locked<Loading<Cover>>> = Shared::default();

                let c = cover.clone();
                let url = i.url.clone();
                tokio::spawn(async move {
                    *c.lock().unwrap() = get_cover(url).await.into();
                });

                cover
            },
        };

        Ok(Self::Artist {
            cover,
            artist,
            top_tracks: api.artist_top_tracks(uri.id(), None).await?,
            albums: pages,
            section: ArtistLanding::default(),
            state: TableState::default(),
            landing_section: LandingSection::default()
        })
    }

    pub fn down(&mut self) {
        match self {
            Landing::Playlist{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Album{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Show{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Audiobook{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Artist{state, section, top_tracks, albums, ..} => match section {
                ArtistLanding::Tracks => {
                    if state.selected().unwrap_or(0) >= top_tracks.len() - 1 {
                        state.select(Some(0));
                        *section = ArtistLanding::Albums;
                    } else {
                        state.next_in_list(top_tracks.len())
                    }
                },
                ArtistLanding::Albums => if let Some(Loading::Some(items)) = albums.items.lock().unwrap().as_ref() {
                    state.next_in_list(items.items.len());
                }
            },
            _ => {},
        }
    }

    pub fn up(&mut self) {
        match self {
            Landing::Playlist{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Album{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Show{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Audiobook{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Artist{state, section, top_tracks, albums, ..} => match section {
                ArtistLanding::Tracks => {
                    state.prev_in_list(top_tracks.len())
                },
                ArtistLanding::Albums => if let Some(Loading::Some(items)) = albums.items.lock().unwrap().as_ref() {
                    if state.selected().unwrap_or(0) == 0 {
                        state.select(Some(top_tracks.len() - 1));
                        *section = ArtistLanding::Tracks;
                    } else {
                        state.prev_in_list(items.items.len());
                    }
                },
            },
            _ => {},
        }
    }

    pub async fn right(&mut self) -> Result<()> {
        match self {
            Landing::Playlist{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Album{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Show{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Audiobook{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Artist{state, section: ArtistLanding::Albums, albums, ..} => if albums.items.lock().unwrap().is_some() && albums.has_next().await {
                albums.next().await?;
                state.select(Some(0));
            }
            _ => {},
        }
        Ok(())
    }

    pub async fn left(&mut self) -> Result<()> {
        match self {
            Landing::Playlist{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Album{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Show{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Audiobook{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Artist{state, section: ArtistLanding::Albums, albums, ..} => if albums.items.lock().unwrap().is_some() && albums.has_prev().await {
                albums.prev().await?;
                state.select(Some(0));
            }
            _ => {},
        }
        Ok(())
    }

    pub fn tab(&mut self) -> Result<()> {
        match self {
            Landing::Playlist { state, section, .. } => {
                state.select(Some(0));
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Album { state, section, .. } => {
                state.select(Some(0));
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Show { state, section, .. } => {
                state.select(Some(0));
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Artist { state, landing_section, .. } => {
                state.select(Some(0));
                *landing_section = LandingSection::from_repr(((*landing_section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Audiobook { state, section, .. } => {
                state.select(Some(0));
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            _ => {}
        }
        Ok(())
    }

    pub fn backtab(&mut self) -> Result<()> {
        match self {
            Landing::Playlist { state, section, .. } => {
                state.select(Some(0));
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Album { state, section, .. } => {
                state.select(Some(0));
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Show { state, section, .. } => {
                state.select(Some(0));
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Artist { state, landing_section, .. } => {
                state.select(Some(0));
                let value = ((*landing_section as isize) - 1) % LandingSection::COUNT as isize;
                *landing_section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Audiobook { state, section, .. } => {
                state.select(Some(0));
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            _ => {}
        }
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<()> {
        match self {
            Landing::Playlist{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() {
                    pages.refresh().await?;
                    state.select(None);
                }
            },
            Landing::Album{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.refresh().await?;
                    state.select(None);
                }
            },
            Landing::Show{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.refresh().await?;
                    state.select(None);
                }
            },
            Landing::Audiobook{ pages, state, .. } => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.refresh().await?;
                    state.select(None);
                }
            },
            Landing::Artist{state, section: ArtistLanding::Albums, albums, ..} => if albums.items.lock().unwrap().is_some() && albums.has_prev().await {
                albums.refresh().await?;
                state.select(None);
            }
            _ => {},
        }
        Ok(())
    }

    pub fn select(&self) -> Option<Vec<(KeyEvent, Action, &'static str)>> {
        match self {
            Landing::Playlist{ playlist, pages, state, section, .. } => {
                return match section {
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        let mut actions = vec![
                            (key!(Enter), Action::PlayContext(Play::playlist(playlist.id.clone(), Some(items.offset + index), 0)), ActionLabel::Play)
                        ];
                        actions.extend(match items.items.get(index) {
                            Some(PlaylistItemInfo { item: Item::Track(t), .. }) => t.into_ui_actions(false),
                            Some(PlaylistItemInfo { item: Item::Episode(e), .. }) => e.into_ui_actions(false),
                            None => return None
                        });
                        Some(actions)
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::playlist(playlist.id.clone(), None, 0)), ActionLabel::Play)
                    ])
                };
            },
            Landing::Album{ album, section, pages, state, .. } => {
                return match section {
                    // Play context from offset instead of playing normally
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        items.items.get(index).map(|t| {
                            let mut actions = vec![
                                (key!(Enter), Action::PlayContext(Play::album(album.id.clone(), Some(items.offset + index), 0)), ActionLabel::Play)
                            ];
                            actions.extend(t.into_ui_actions(false));
                            actions
                        })
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::album(album.id.clone(), None, 0)), ActionLabel::Play)
                    ]),
                };
            },
            Landing::Show{ show, section, pages, state, .. } => {
                return match section {
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        items.items.get(index).map(|t| {
                            let mut actions = vec![
                                (key!(Enter), Action::PlayContext(Play::show(show.id.clone(), Some(items.offset + index), 0)), ActionLabel::Play)
                            ];
                            actions.extend(t.into_ui_actions(false));
                            actions
                        })
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::show(show.id.clone(), None, 0)), ActionLabel::Play)
                    ])
                }
            },
            Landing::Audiobook{ audiobook, section, pages, state, .. } => {
                return match section {
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        items.items.get(index).map(|t| {
                            let mut actions = vec![
                                (key!(Enter), Action::PlayContext(Play::show(audiobook.id.clone(), Some(items.offset + index), 0)), ActionLabel::Play)
                            ];
                            actions.extend(t.into_ui_actions(false));
                            actions
                        })
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::show(audiobook.id.clone(), None, 0)), ActionLabel::Play)
                    ])
                }
            },
            Landing::Artist{state, section, top_tracks, albums, landing_section, artist, ..} => {
                return match landing_section {
                    LandingSection::Content => match section {
                        ArtistLanding::Albums => if let Some(Loading::Some(items)) = albums.items.lock().unwrap().as_ref() {
                            let index = state.selected().unwrap_or(0);
                            items.items.get(index).map(|t| t.into_ui_actions(false))
                        } else {
                            None
                        },
                        ArtistLanding::Tracks => {
                            let index = state.selected().unwrap_or(0);
                            return top_tracks.get(index).map(|t| t.into_ui_actions(false))
                        },
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::artist(artist.id.clone())), ActionLabel::Play),
                        // TODO: Need to know if already saved: Use save or remove depending on
                        // state
                        (key!('f'), Action::Save(artist.uri.clone()), ActionLabel::Save),
                    ])
                }
            },
            _ => {},
        }
        None
    }
}
