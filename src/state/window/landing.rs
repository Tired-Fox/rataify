use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::widgets::TableState;
use tupy::{api::{response::{PlaylistItems, AlbumTracks, Item, PlaylistItemInfo, Chapters, ShowEpisodes}, Uri, PublicApi, flow::Pkce}};

use super::Pages;
use crate::{state::{IterCollection, Loading}, ui::{Action, IntoActions}};

#[derive(Default, Debug, Clone)]
pub enum Landing {
    #[default]
    None,
    Artist,
    Playlist(Pages<PlaylistItems, PlaylistItems>, TableState),
    Album(Pages<AlbumTracks, AlbumTracks>, TableState),
    Show(Pages<ShowEpisodes, ShowEpisodes>, TableState),
    Audiobook(Pages<Chapters, Chapters>, TableState),
}

impl Landing {
    pub async fn playlist(api: &Pkce, playlist: Uri) -> Result<Self> {
        let mut pages = Pages::new(api.playlist_items(playlist, None)?);
        pages.next().await?;
        Ok(Self::Playlist(pages, TableState::default()))
    }

    pub async fn album(api: &Pkce, album: Uri) -> Result<Self> {
        let mut pages = Pages::new(api.album_tracks(album, None)?);
        pages.next().await?;
        Ok(Self::Album(pages, TableState::default()))
    }

    pub async fn show(api: &Pkce, show: Uri) -> Result<Self> {
        let mut pages = Pages::new(api.show_episodes(show, None)?);
        pages.next().await?;
        Ok(Self::Show(pages, TableState::default()))
    }

    pub async fn audiobook(api: &Pkce, audiobook: Uri) -> Result<Self> {
        let mut pages = Pages::new(api.audiobook_chapters(audiobook, None)?);
        pages.next().await?;
        Ok(Self::Audiobook(pages, TableState::default()))
    }

    pub fn down(&mut self) {
        match self {
            Landing::Playlist(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Album(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Show(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            Landing::Audiobook(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.next_in_list(items.items.len());
            },
            _ => {},
        }
    }

    pub fn up(&mut self) {
        match self {
            Landing::Playlist(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Album(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Show(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            Landing::Audiobook(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                state.prev_in_list(items.items.len());
            },
            _ => {},
        }
    }

    pub async fn right(&mut self) -> Result<()> {
        match self {
            Landing::Playlist(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Album(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Show(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            Landing::Audiobook(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_next().await {
                    pages.next().await?;
                    state.select(Some(0));
                }
            },
            _ => {},
        }
        Ok(())
    }

    pub async fn left(&mut self) -> Result<()> {
        match self {
            Landing::Playlist(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Album(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Show(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            Landing::Audiobook(pages, state) => {
                if pages.items.lock().unwrap().is_some() && pages.has_prev().await {
                    pages.prev().await?;
                    state.select(Some(0));
                }
            },
            _ => {},
        }
        Ok(())
    }

    pub fn select(&self) -> Option<Vec<(KeyEvent, Action)>> {
        match self {
            Landing::Playlist(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                return match items.items.get(state.selected().unwrap_or(0)) {
                    Some(PlaylistItemInfo { item: Item::Track(t), .. }) => Some(t.into_ui_actions(false)),
                    Some(PlaylistItemInfo { item: Item::Episode(e), .. }) => Some(e.into_ui_actions(false)),
                    None => None
                }
            },
            Landing::Album(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                return items.items.get(state.selected().unwrap_or(0)).map(|t| t.into_ui_actions(false))
            },
            Landing::Show(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                return items.items.get(state.selected().unwrap_or(0)).map(|t| t.into_ui_actions(false))
            },
            Landing::Audiobook(pages, state) => if let Some(Loading::Some(items)) = pages.items.lock().unwrap().as_ref() {
                return items.items.get(state.selected().unwrap_or(0)).map(|t| t.into_ui_actions(false))
            },
            _ => {},
        }
        None
    }
}
