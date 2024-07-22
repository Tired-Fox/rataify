use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, widgets::TableState};
use ratatui_image::{picker::Picker, protocol::Protocol, Resize};
use strum::EnumCount;
use tupy::api::{flow::Pkce, request::{IncludeGroup, Play}, response::{Album, AlbumTracks, Artist, ArtistAlbums, Audiobook, Chapters, Item, Playlist, PlaylistItemInfo, PlaylistItems, Show, ShowEpisodes, SimplifiedAlbum, SimplifiedEpisode, SimplifiedTrack, Track}, PublicApi, Uri, UserApi};

use super::{MappedPages, Pages};
use crate::{errors::LogError, key, state::{actions::{action_label, Action, IntoActions}, wrappers::{GetUri, Saved}, IterCollection, Loading}, Locked, Shared};

#[derive(Default, Debug, Clone, Copy, strum_macros::EnumIs)]
pub enum ArtistLanding {
    #[default]
    Tracks,
    Albums,
}

pub struct Cover {
    pub picker: Picker,
    pub image: Box<dyn Protocol>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, strum_macros::EnumIs, strum_macros::EnumIter, strum_macros::FromRepr, strum_macros::EnumCount)]
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
        artist: Shared<Locked<Saved<Artist>>>,
        top_tracks: Shared<Locked<Vec<Saved<Track>>>>,
        albums: MappedPages<Vec<Saved<SimplifiedAlbum>>, ArtistAlbums, ArtistAlbums>,

        section: ArtistLanding,
        state: TableState,
        landing_section: LandingSection,
    },
    Playlist {
        cover: Shared<Locked<Loading<Cover>>>,
        playlist: Playlist,
        pages: MappedPages<Vec<Saved<PlaylistItemInfo>>, PlaylistItems, PlaylistItems>,
        state: TableState,
        section: LandingSection,
    },
    Album {
        cover: Shared<Locked<Loading<Cover>>>,
        album: Album,
        pages: MappedPages<Vec<Saved<SimplifiedTrack>>, AlbumTracks, AlbumTracks>,
        state: TableState,
        section: LandingSection,
    },
    Show {
        cover: Shared<Locked<Loading<Cover>>>,
        show: Show,
        pages: MappedPages<Vec<Saved<SimplifiedEpisode>>, ShowEpisodes, ShowEpisodes>,
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
        let pages = MappedPages::new(
            api.playlist_items(playlist.id(), None)?,
            |data, api| Box::pin(async move {
                Ok(match data {
                    Some(data) => {
                        // Saved tracks
                        let mut tracks = api.check_saved_tracks(data.items.iter().filter_map(|v| match &v.item {
                            Item::Track(t) => Some(t.id.clone()),
                            _ => None
                        })).await.log_error_or(vec![]).into_iter();

                        // Saved episodes
                        let mut episodes = api.check_saved_episodes(data.items.iter().filter_map(|v| match &v.item {
                            Item::Episode(e) => Some(e.id.clone()),
                            _ => None
                        })).await.log_error_or(vec![]).into_iter();

                        // Map to saved items popping from appropriate list
                        let items = data.items.into_iter().map(|i| Saved::new(match i.item {
                            Item::Track(_) => tracks.next().unwrap_or_default(),
                            Item::Episode(_) => episodes.next().unwrap_or_default(),
                        }, i)).collect();

                        Some(items)
                    },
                    None => None
                })
            })
        );

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.log_error();
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
        let pages = MappedPages::new(
            api.album_tracks(album.id(), None)?,
            |data, api| Box::pin(async move {
                Ok(match data {
                    Some(data) => {
                        let tracks = api.check_saved_tracks(data.items.iter().map(|t| t.id.clone())).await.log_error_or(vec![false]);
                        Some(data.items.into_iter().zip(tracks.into_iter()).map(|(i, s)| Saved::new(s, i)).collect())
                    },
                    None => None
                })
            })
        );

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.log_error();
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
        let pages = MappedPages::new(
            api.show_episodes(show.id(), None)?,
            |data, api| Box::pin(async move {
                Ok(match data {
                    Some(data) => {
                        let checked = api.check_saved_episodes(data.items.iter().map(|v| v.id.clone())).await?;
                        Some(data.items.into_iter().zip(checked.into_iter()).map(|(d, s)| Saved::new(s, d)).collect())
                    },
                    None => None
                })
            })
        );

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.log_error();
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
            p.next().await.log_error();
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
        let pages = MappedPages::new(
            api.artist_albums(uri.id(), None, &[IncludeGroup::Single, IncludeGroup::Album, IncludeGroup::AppearsOn])?,
            |data, api| Box::pin(async move {
                Ok(match data {
                    Some(data) => {
                        let checked = api.check_saved_albums(data.items.iter().map(|v| v.id.clone())).await?;
                        Some(data.items.into_iter().zip(checked.into_iter()).map(|(d, s)| Saved::new(s, d)).collect())
                    },
                    None => None
                })
            })
        );

        let p = pages.clone();
        tokio::spawn(async move {
            p.next().await.log_error();
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

        let top_tracks = api.artist_top_tracks(uri.id(), None).await?;
        let saved_top_tracks = api.check_saved_tracks(top_tracks.iter().map(|t| t.id.clone())).await?;

        Ok(Self::Artist {
            cover,
            artist: Shared::new(Locked::new(Saved::new(
                api.check_follow_artists([artist.id.clone()]).await?[0],
                artist
            ))),
            top_tracks: Shared::new(Locked::new(top_tracks.into_iter().zip(saved_top_tracks.into_iter()).map(|(t, s)| Saved::new(s, t)).collect())),
            albums: pages,
            section: ArtistLanding::default(),
            state: TableState::default(),
            landing_section: LandingSection::default()
        })
    }

    pub fn down(&mut self) {
        match self {
            Landing::Playlist{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.len());
            },
            Landing::Album{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.len());
            },
            Landing::Show{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.len());
            },
            Landing::Audiobook{pages, state, ..} => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Artist{state, section, top_tracks, albums, ..} => match section {
                ArtistLanding::Tracks => {
                    let top_tracks = top_tracks.lock().unwrap();
                    if state.selected().unwrap_or(0) >= top_tracks.len() - 1 {
                        state.select(Some(0));
                        *section = ArtistLanding::Albums;
                    } else {
                        state.next_in_list(top_tracks.len())
                    }
                },
                ArtistLanding::Albums => if let Some(Loading::Some(items)) = albums.items.lock().unwrap().as_ref() {
                    state.next_in_list(items.len());
                }
            },
            _ => {},
        }
    }

    pub fn up(&mut self) {
        match self {
            Landing::Playlist{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.len());
            },
            Landing::Album{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.len());
            },
            Landing::Show{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.len());
            },
            Landing::Audiobook{ pages, state, .. } => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Artist{state, section, top_tracks, albums, ..} => match section {
                ArtistLanding::Tracks => {
                    state.prev_in_list(top_tracks.lock().unwrap().len())
                },
                ArtistLanding::Albums => if let Some(Loading::Some(items)) = albums.items.lock().unwrap().as_ref() {
                    if state.selected().unwrap_or(0) == 0 {
                        state.select(Some(top_tracks.lock().unwrap().len() - 1));
                        *section = ArtistLanding::Tracks;
                    } else {
                        state.prev_in_list(items.len());
                    }
                },
            },
            _ => {},
        }
    }

    pub async fn right(&mut self) -> Result<()> {
        match self {
            Landing::Playlist{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Album{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Show{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Audiobook{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Artist{ state, section: ArtistLanding::Albums, albums, landing_section, .. } if landing_section.is_content() => if albums.items.lock().unwrap().is_some() && albums.has_next().await {
                albums.next().await?;
                state.select(Some(0));
            }
            _ => {},
        }
        Ok(())
    }

    pub async fn left(&mut self) -> Result<()> {
        match self {
            Landing::Playlist{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Album{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Show{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Audiobook{ pages, state, section, .. } if section.is_content() => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Artist{state, section: ArtistLanding::Albums, albums, landing_section, ..} if landing_section.is_content() => if albums.items.lock().unwrap().is_some() && albums.has_prev().await {
                albums.prev().await?;
                state.select(Some(0));
            }
            _ => {},
        }
        Ok(())
    }

    pub fn tab(&mut self) -> Result<()> {
        match self {
            Landing::Playlist { section, .. } => {
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Album { section, .. } => {
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Show { section, .. } => {
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Artist { landing_section, .. } => {
                *landing_section = LandingSection::from_repr(((*landing_section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            Landing::Audiobook { section, .. } => {
                *section = LandingSection::from_repr(((*section as usize) + 1) % LandingSection::COUNT).unwrap();
            },
            _ => {}
        }
        Ok(())
    }

    pub fn backtab(&mut self) -> Result<()> {
        match self {
            Landing::Playlist { section, .. } => {
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Album { section, .. } => {
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Show { section, .. } => {
                let value = ((*section as isize) - 1) % LandingSection::COUNT as isize;
                *section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Artist { landing_section, .. } => {
                let value = ((*landing_section as isize) - 1) % LandingSection::COUNT as isize;
                *landing_section = LandingSection::from_repr(value as usize).unwrap();
            },
            Landing::Audiobook { section, .. } => {
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
                let i = pages.items.clone();
                return match section {
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        let mut actions = vec![
                            (key!(Enter), Action::PlayContext(Play::playlist(playlist.id.clone(), Some(pages.page.lock().unwrap().offset + index), 0)), action_label::PLAY)
                        ];
                        actions.extend(match items.get(index) {
                            Some(saved) => saved.into_actions(false, move |saved| {
                                if let Some(Loading::Some(items)) = i.lock().unwrap().as_mut().map(|v| v.as_mut()) {
                                    items[index].saved = saved;
                                }
                                Ok(())
                            }),
                            None => return None
                        });
                        Some(actions)
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::playlist(playlist.id.clone(), None, 0)), action_label::PLAY)
                    ])
                };
            },
            Landing::Album{ album, section, pages, state, .. } => {
                let i = pages.items.clone();
                return match section {
                    // Play context from offset instead of playing normally
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        let page = pages.page.lock().unwrap();
                        items.get(index).map(|t| {
                            let mut actions = vec![
                                (key!(Enter), Action::PlayContext(Play::album(album.id.clone(), Some(page.offset + index), 0)), action_label::PLAY)
                            ];
                            actions.extend(t.into_actions(false, move |saved| {
                                if let Some(Loading::Some(items)) = i.lock().unwrap().as_mut().map(|v| v.as_mut()) {
                                    items[index].saved = saved;
                                }
                                Ok(())
                            }));
                            actions
                        })
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::album(album.id.clone(), None, 0)), action_label::PLAY)
                    ]),
                };
            },
            Landing::Show{ show, section, pages, state, .. } => {
                let i = pages.items.clone();
                return match section {
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let page = pages.page.lock().unwrap();
                        let index = state.selected().unwrap_or(0);
                        items.get(index).map(|e| {
                            let mut actions = vec![
                                (key!(Enter), Action::PlayContext(Play::show(show.id.clone(), Some(page.offset + index), 0)), action_label::PLAY)
                            ];
                            actions.extend(e.into_actions(false, move |saved| {
                                if let Some(Loading::Some(items)) = i.lock().unwrap().as_mut().map(|v| v.as_mut()) {
                                    items[index].saved = saved;
                                }
                                Ok(())
                            }));
                            actions
                        })
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::show(show.id.clone(), None, 0)), action_label::PLAY)
                    ])
                }
            },
            Landing::Audiobook{ audiobook, section, pages, state, .. } => {
                return match section {
                    LandingSection::Content => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                        let index = state.selected().unwrap_or(0);
                        items.items.get(index).map(|t| {
                            let mut actions = vec![
                                (key!(Enter), Action::PlayContext(Play::show(audiobook.id.clone(), Some(items.offset + index), 0)), action_label::PLAY)
                            ];
                            actions.extend(t.into_actions(false));
                            actions
                        })
                    } else {
                        None
                    },
                    LandingSection::Context => Some(vec![
                        (key!(Enter), Action::PlayContext(Play::show(audiobook.id.clone(), None, 0)), action_label::PLAY)
                    ])
                }
            },
            Landing::Artist{state, section, top_tracks, albums, landing_section, artist, ..} => {
                return match landing_section {
                    LandingSection::Content => match section {
                        ArtistLanding::Albums => {
                            let i = albums.items.clone();
                            if let Some(Loading::Some(items)) = albums.items.lock().unwrap().as_ref() {
                                let index = state.selected().unwrap_or(0);
                                items.get(index).map(|t| {
                                    let mut actions = vec![
                                        (key!(Enter), Action::PlayContext(Play::album(t.as_ref().id.clone(), None, 0)), action_label::PLAY)
                                    ];
                                    actions.extend(t.into_actions(false, move |saved| {
                                        if let Some(Loading::Some(items)) = i.lock().unwrap().as_mut().map(|v| v.as_mut()) {
                                            items[index].saved = saved;
                                        } 
                                        Ok(())
                                    }));
                                    actions
                                })
                            } else {
                                None
                            }
                        }
                        ArtistLanding::Tracks => {
                            let index = state.selected().unwrap_or(0);
                            let tt = top_tracks.clone();
                            return top_tracks.lock().unwrap().get(index).map(|t| {
                                let mut actions = vec![
                                    (key!(Enter), Action::Play(t.as_ref().uri.clone()), action_label::PLAY)
                                ];
                                actions.extend(t.into_actions(false, move |saved| {
                                    tt.lock().unwrap()[index].saved = saved;
                                    Ok(())
                                }));
                                actions
                            })
                        },
                    },
                    LandingSection::Context => {
                        let a = artist.clone();
                        let update_saved = move |saved| {
                            a.lock().unwrap().saved = saved;
                            Ok(())
                        };
                        let artist = artist.lock().unwrap();
                        Some(vec![
                            (key!(Enter), Action::PlayContext(Play::artist(artist.as_ref().id.clone())), action_label::PLAY),
                            if artist.saved {
                                (key!('r'), Action::remove(artist.as_ref().uri.clone(), update_saved), action_label::REMOVE)
                            } else {
                                (key!('f'), Action::save(artist.as_ref().uri.clone(), update_saved), action_label::SAVE)
                            },
                        ])
                    }
                }
            },
            _ => {},
        }
        None
    }
}
