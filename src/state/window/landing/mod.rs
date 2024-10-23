use album::AlbumDetails;
use artist::ArtistDetails;
use image::ImageFormat;
use playlist::PlaylistDetails;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use rspotify::{
    clients::BaseClient,
    model::{AdditionalType, Id},
    AuthCodePkceSpotify,
};
use show::ShowDetails;

use crate::{
    action::{Action, Offset, Open},
    app::ContextSender,
    key,
    state::{
        model::{Album, Artist, Item, Playlist, Show, Track},
        ActionList, InnerState, Loadable,
    },
    ConvertPage, Error,
};

pub mod album;
pub mod artist;
pub mod playlist;
pub mod show;

#[derive(Debug, Clone)]
pub enum LandingState {
    Playlist(PlaylistDetails),
    Album(AlbumDetails),
    Artist(ArtistDetails),
    Show(ShowDetails),
}

impl LandingState {
    pub async fn handle_action(
        &mut self,
        action: Action,
        spotify: &AuthCodePkceSpotify,
        state: &InnerState,
        sender: ContextSender,
    ) -> Result<(), Error> {
        match action {
            Action::Up => match self {
                Self::Playlist(playlist) => playlist.index = playlist.index.saturating_sub(1),
                Self::Album(album) => album.index = album.index.saturating_sub(1),
                Self::Artist(artist) => artist.index = artist.index.saturating_sub(1),
                Self::Show(show) => show.index = show.index.saturating_sub(1),
            },
            Action::Down => match self {
                Self::Playlist(playlist) => {
                    if playlist.index < (playlist.items.limit as usize - 1) {
                        playlist.index += 1
                    }
                }
                Self::Album(album) => {
                    if album.index < (album.tracks.limit as usize - 1) {
                        album.index += 1
                    }
                }
                Self::Artist(artist) => {
                    if artist.index < ((artist.albums.limit as usize + artist.top_tracks.len()) - 1)
                    {
                        artist.index += 1
                    }
                }
                Self::Show(show) => {
                    if show.index < (show.episodes.limit as usize - 1) {
                        show.index += 1
                    }
                }
            },
            Action::Select => match self {
                Self::Playlist(playlist) => match playlist.items.items.get(playlist.index) {
                    Some(Item::Track(track)) => sender.send_action(Action::Open(Open::actions(
                        track.action_list_with(
                            [(
                                key!(Enter),
                                playlist
                                    .playlist
                                    .play(Some(Offset::Position(playlist.items.offset as usize + playlist.index))),
                            )],
                            true,
                        ),
                    )))?,
                    Some(Item::Episode(episode)) => {
                        sender.send_action(Action::Open(Open::actions(
                            episode.action_list_with(
                                [(
                                    key!(Enter),
                                    playlist
                                        .playlist
                                        .play(Some(Offset::Uri(episode.id.clone().uri()))),
                                )],
                                true,
                            ),
                        )))?
                    }
                    _ => {}
                },
                Self::Album(album) => {
                    if let Some(track) = album.tracks.items.get(album.index) {
                        sender.send_action(Action::Open(Open::actions(
                            track.action_list_with(
                                [(
                                    key!(Enter),
                                    album
                                        .album
                                        .play(Some(Offset::Position(album.tracks.offset as usize + album.index))),
                                )],
                                true,
                            ),
                        )))?;
                    }
                }
                Self::Artist(artist) => {
                    if artist.index < 10 {
                        if let Some(track) = artist.top_tracks.get(artist.index) {
                            sender.send_action(Action::Open(Open::actions(
                                track.action_list_with([
                                    (key!({CONTROL}Enter), artist.artist.play()),
                                    (key!(Enter), track.play()),
                                ], true),
                            )))?;
                        }
                    } else if let Some(album) =
                        artist.albums.items.get(artist.index.saturating_sub(10))
                    {
                        sender.send_action(Action::Open(Open::actions(album.action_list_with(
                            [(key!({CONTROL}Enter), artist.artist.play())],
                            true,
                        ))))?;
                    }
                }
                Self::Show(show) => {
                    if let Some(episode) = show.episodes.items.get(show.index) {
                        sender.send_action(Action::Open(Open::actions(
                            episode
                                .action_list_with([(key!(Enter), show.show.play(Some(Offset::Uri(episode.id.clone().uri()))))], true),
                        )))?;
                    }
                }
            },
            Action::NextPage => match self {
                Self::Playlist(playlist) => {
                    if playlist.items.offset + playlist.items.limit < playlist.items.total - 1 {
                        let limit = playlist.items.limit;
                        let offset = (playlist.items.offset + playlist.items.limit)
                            .min(playlist.items.total);
                        let id = playlist.playlist.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .playlist_items_manual(
                                    id,
                                    None,
                                    None,
                                    Some(limit),
                                    Some(offset),
                                    Some(&[AdditionalType::Track, AdditionalType::Episode]),
                                )
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Playlist(playlist)) =
                                landing.lock().unwrap().as_mut()
                            {
                                playlist.items = result;
                            }
                        });
                    }
                }
                Self::Album(album) => {
                    if album.tracks.offset + album.tracks.limit < album.tracks.total - 1 {
                        let limit = album.tracks.limit;
                        let offset =
                            (album.tracks.offset + album.tracks.limit).min(album.tracks.total);
                        let id = album.album.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .album_track_manual(id, None, Some(limit), Some(offset))
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Album(album)) =
                                landing.lock().unwrap().as_mut()
                            {
                                album.tracks = result;
                            }
                        });
                    }
                }
                Self::Artist(artist) => {
                    if artist.albums.offset + artist.albums.limit < artist.albums.total - 1 {
                        let limit = artist.albums.limit;
                        let offset =
                            (artist.albums.offset + artist.albums.limit).min(artist.albums.total);
                        let id = artist.artist.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .artist_albums_manual(
                                    id.clone(),
                                    [],
                                    None,
                                    Some(limit),
                                    Some(offset),
                                )
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Artist(artist)) =
                                landing.lock().unwrap().as_mut()
                            {
                                artist.albums = result;
                            }
                        });
                    }
                }
                Self::Show(show) => {
                    if show.episodes.offset + show.episodes.limit < show.episodes.total - 1 {
                        let limit = show.episodes.limit;
                        let offset =
                            (show.episodes.offset + show.episodes.limit).min(show.episodes.total);
                        let id = show.show.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .get_shows_episodes_manual(
                                    id.clone(),
                                    None,
                                    Some(limit),
                                    Some(offset),
                                )
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Show(show)) =
                                landing.lock().unwrap().as_mut()
                            {
                                show.episodes = result;
                            }
                        });
                    }
                }
            },
            Action::PreviousPage => match self {
                Self::Playlist(playlist) => {
                    if playlist.items.offset > 0 {
                        let limit = playlist.items.limit;
                        let offset = playlist.items.offset.saturating_sub(playlist.items.limit);
                        let id = playlist.playlist.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .playlist_items_manual(
                                    id,
                                    None,
                                    None,
                                    Some(limit),
                                    Some(offset),
                                    Some(&[AdditionalType::Track, AdditionalType::Episode]),
                                )
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Playlist(playlist)) =
                                landing.lock().unwrap().as_mut()
                            {
                                playlist.items = result;
                            }
                        });
                    }
                }
                Self::Album(album) => {
                    if album.tracks.offset > 0 {
                        let limit = album.tracks.limit;
                        let offset = album.tracks.offset.saturating_sub(album.tracks.limit);
                        let id = album.album.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .album_track_manual(id, None, Some(limit), Some(offset))
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Album(album)) =
                                landing.lock().unwrap().as_mut()
                            {
                                album.tracks = result;
                            }
                        });
                    }
                }
                Self::Artist(artist) => {
                    if artist.albums.offset > 0 {
                        let limit = artist.albums.limit;
                        let offset = artist.albums.offset.saturating_sub(artist.albums.limit);
                        let id = artist.artist.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .artist_albums_manual(
                                    id.clone(),
                                    [],
                                    None,
                                    Some(limit),
                                    Some(offset),
                                )
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Artist(artist)) =
                                landing.lock().unwrap().as_mut()
                            {
                                artist.albums = result;
                            }
                        });
                    }
                }
                Self::Show(show) => {
                    if show.episodes.offset > 0 {
                        let limit = show.episodes.limit;
                        let offset = show.episodes.offset.saturating_sub(show.episodes.limit);
                        let id = show.show.id.clone();
                        let landing = state.landing.clone();

                        let spot = spotify.clone();
                        tokio::spawn(async move {
                            let result = spot
                                .get_shows_episodes_manual(
                                    id.clone(),
                                    None,
                                    Some(limit),
                                    Some(offset),
                                )
                                .await
                                .unwrap()
                                .convert_page();

                            if let Loadable::Some(LandingState::Show(show)) =
                                landing.lock().unwrap().as_mut()
                            {
                                show.episodes = result;
                            }
                        });
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct Landing;
impl StatefulWidget for Landing {
    type State = InnerState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let block = Block::default();

        if let Some(landing) = state
            .landing
            .lock()
            .unwrap()
            .render_unwrap_mut(block.inner(area), buf)
        {
            match landing {
                LandingState::Playlist(playlist) => playlist.render(block.inner(area), buf),
                LandingState::Album(album) => album.render(block.inner(area), buf),
                LandingState::Artist(artist) => artist.render(block.inner(area), buf),
                LandingState::Show(show) => show.render(block.inner(area), buf),
            }
        }
    }
}

impl LandingState {
    pub async fn get_playlist(
        playlist: Playlist,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Playlist(PlaylistDetails {
            index: 0,
            image,
            items: spotify
                .playlist_items_manual(
                    playlist.id.clone(),
                    None,
                    None,
                    Some(30),
                    Some(0),
                    Some(&[AdditionalType::Track, AdditionalType::Episode]),
                )
                .await?
                .convert_page(),
            playlist,
        }))
    }

    pub async fn get_album(
        album: Album,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Album(AlbumDetails {
            index: 0,
            image,
            tracks: spotify
                .album_track_manual(album.id.clone(), None, Some(30), Some(0))
                .await?
                .convert_page(),
            album,
        }))
    }

    pub async fn get_show(
        show: Show,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Show(ShowDetails {
            index: 0,
            image,
            episodes: spotify
                .get_shows_episodes_manual(show.id.clone(), None, Some(30), Some(0))
                .await?
                .convert_page(),
            show,
        }))
    }

    pub async fn get_artist(
        artist: Artist,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Artist(ArtistDetails {
            index: 0,
            image,
            albums: spotify
                .artist_albums_manual(artist.id.clone(), [], None, Some(20), Some(0))
                .await?
                .convert_page(),
            top_tracks: spotify
                .artist_top_tracks(artist.id.clone(), None)
                .await?
                .into_iter()
                .map(Track::from)
                .collect(),
            artist,
        }))
    }
}

async fn fetch_image(image: String) -> Option<Box<dyn StatefulProtocol>> {
    let mut picker = Picker::new((7, 16));
    let image = reqwest::get(image.as_str())
        .await
        .ok()?
        .bytes()
        .await
        .ok()?
        .to_vec();
    let dyn_image = image::load_from_memory_with_format(&image, ImageFormat::Jpeg).ok()?;
    Some(picker.new_resize_protocol(dyn_image))
}
