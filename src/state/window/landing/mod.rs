use album::AlbumDetails;
use artist::ArtistDetails;
use image::ImageFormat;
use playlist::PlaylistDetails;
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use rspotify::{
    clients::BaseClient,
    model::{
        AdditionalType, AlbumId, ArtistId, PlaylistId, ShowId
    },
    AuthCodePkceSpotify,
};
use show::ShowDetails;

use crate::{action::Action, app::ContextSender, state::{model::Track, InnerState}, ConvertPage, Error};

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
    pub fn handle_action(
        &mut self,
        action: Action,
        spotify: &AuthCodePkceSpotify,
        state: &InnerState,
        sender: ContextSender,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub struct Landing;
impl StatefulWidget for Landing {
    type State = InnerState;
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let block = Block::default();

        if let Some(landing) = state.landing.lock().unwrap().render_unwrap_mut(block.inner(area), buf) {
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
        playlist: PlaylistId<'static>,
        name: String,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Playlist(PlaylistDetails {
            image,
            name,
            items: spotify
                .playlist_items_manual(playlist.clone(), None, None, Some(10), Some(0), Some(&[AdditionalType::Track, AdditionalType::Episode]))
                .await?
                .convert_page(),
        }))
    }

    pub async fn get_album(
        album: AlbumId<'static>,
        name: String,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Album(AlbumDetails{
            image,
            name,
            tracks: spotify
                .album_track_manual(album.clone(), None, Some(10), Some(0))
                .await?
                .convert_page(),
        }))
    }

    pub async fn get_show(
        show: ShowId<'static>,
        name: String,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Show(ShowDetails{
            image,
            name,
            episodes: spotify
                .get_shows_episodes_manual(show.clone(), None, Some(10), Some(0))
                .await?
                .convert_page(),
        }))
    }

    pub async fn get_artist(
        artist: ArtistId<'static>,
        name: String,
        image: Option<String>,
        spotify: AuthCodePkceSpotify,
    ) -> Result<Self, Error> {
        let image = match image {
            Some(img) => fetch_image(img).await,
            None => None,
        };

        Ok(Self::Artist(ArtistDetails{
            image,
            name,
            albums: spotify
                .artist_albums_manual(artist.clone(), [], None, Some(20), Some(0))
                .await?
                .convert_page(),
            top_tracks: spotify.artist_top_tracks(artist.clone(), None).await?.into_iter().map(Track::from).collect(),
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
