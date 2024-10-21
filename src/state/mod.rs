mod playback;
pub mod window;

use std::{
    any::type_name,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::Future;
use playback::Playback;
use ratatui::{
    layout::{Constraint, Layout},
    text::Line,
    widgets::{Clear, Paragraph, StatefulWidget, Widget},
};
use rspotify::{
    clients::OAuthClient,
    model::{AdditionalType, Device},
    scopes, AuthCodePkceSpotify, Credentials, OAuth,
};
use window::{
    library::LibraryState,
    modal::{device::DeviceState, Modal},
    Window,
};

use crate::{action::Action, api, app::ContextSender, ConvertPage, Error};

#[derive(Default, strum::EnumIs)]
pub enum Loadable<T> {
    #[default]
    None,
    Loading,
    Some(T),
}

impl<T: std::fmt::Debug> std::fmt::Debug for Loadable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Load::None"),
            Self::Loading => write!(f, "Load::Loading"),
            Self::Some(t) => write!(f, "Load::Some({t:?})"),
        }
    }
}

impl<T: Clone> Clone for Loadable<T> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Loading => Self::Loading,
            Self::Some(t) => Self::Some(t.clone()),
        }
    }
}

impl<T: Copy> Copy for Loadable<T> {}

impl<T> Loadable<T> {
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Self::None => panic!("cannot unwrap Load<{}>; no value", type_name::<T>()),
            Self::Loading => panic!(
                "cannot unwrap Load<{}>; value was loading",
                type_name::<T>()
            ),
            Self::Some(t) => t,
        }
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_ref(&self) -> Loadable<&T> {
        match self {
            Self::None => Loadable::None,
            Self::Loading => Loadable::Loading,
            Self::Some(t) => Loadable::Some(t),
        }
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_mut(&mut self) -> Loadable<&mut T> {
        match self {
            Self::None => Loadable::None,
            Self::Loading => Loadable::Loading,
            Self::Some(t) => Loadable::Some(t),
        }
    }

    #[inline]
    pub fn replace(&mut self, new: T) -> Loadable<T> {
        std::mem::replace(self, Self::Some(new))
    }

    #[inline]
    pub fn take(&mut self) -> Loadable<T> {
        std::mem::replace(self, Self::None)
    }

    #[inline]
    pub fn load(&mut self) -> Loadable<T> {
        std::mem::replace(self, Self::Loading)
    }
}

trait Subscriber {
    fn call(
        &self,
        spotify: AuthCodePkceSpotify,
        inner: InnerState,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>;
}

impl<F, Fut> Subscriber for F
where
    F: Fn(AuthCodePkceSpotify, InnerState) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<(), Error>>,
{
    fn call(
        &self,
        spotify: AuthCodePkceSpotify,
        inner: InnerState,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        let handler = self.clone();
        Box::pin(async move { handler(spotify, inner).await })
    }
}

#[derive(Clone)]
struct Timer {
    target: Duration,
    current: Duration,
    action: Rc<dyn Subscriber>,
}

impl Timer {
    pub fn new<F>(target: Duration, action: F) -> Self
    where
        F: Subscriber + 'static,
    {
        Self {
            target,
            current: Duration::default(),
            action: Rc::new(action),
        }
    }

    pub async fn tick(
        &mut self,
        dt: Duration,
        spotify: &AuthCodePkceSpotify,
        state: &InnerState,
    ) -> Result<(), Error> {
        self.current += dt;
        if self.current >= self.target {
            self.action.call(spotify.clone(), state.clone()).await?;
            self.current = Duration::default();
        }
        Ok(())
    }
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("target", &self.target)
            .field("current", &self.current)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct State {
    playback_ping: Timer,
    pub spotify: AuthCodePkceSpotify,
    pub inner: InnerState,
}

impl State {
    pub async fn new(inner: InnerState, ctx: ContextSender) -> Result<Self, Error> {
        let cache_path = dirs::cache_dir().unwrap().join("rataify");

        if !cache_path.exists() {
            std::fs::create_dir_all(&cache_path)?;
        }

        let mut spotify = AuthCodePkceSpotify::with_config(
            Credentials::from_env().ok_or(Error::custom(
                "failed to parse spotify Credentials from environment variables",
            ))?,
            OAuth::from_env(scopes![
                "user-read-playback-state",
                "user-follow-read",
                "playlist-read-private",
                "user-library-read"
            ])
            .ok_or(Error::custom(
                "failed to parse spotify OAuth from environment variables",
            ))?,
            rspotify::Config {
                cache_path: cache_path.join("token.json"),
                token_cached: true,
                token_refreshing: true,
                ..Default::default()
            },
        );

        let url = spotify.get_authorize_url(None)?;
        spotify.prompt_for_token(url.as_str()).await?;

        inner.init(&spotify, ctx)?;

        Ok(Self {
            playback_ping: Timer::new(
                Duration::from_secs(5),
                |spotify: AuthCodePkceSpotify, inner: InnerState| async move {
                    let playback = inner.playback.clone();
                    tokio::spawn(async move {
                        let ctx = spotify
                            .current_playback(
                                None,
                                Some(vec![&AdditionalType::Track, &AdditionalType::Episode]),
                            )
                            .await
                            .unwrap()
                            .map(Playback::from);
                        *playback.lock().unwrap() = ctx;
                    });

                    Ok(())
                },
            ),

            spotify,
            inner,
        })
    }

    pub async fn tick(&mut self, dt: Duration) -> Result<(), Error> {
        self.playback_ping
            .tick(dt, &self.spotify, &self.inner)
            .await
    }

    pub fn close_modal(&self) -> bool {
        return self.inner.modal.lock().unwrap().take().is_none();
    }

    pub fn playing(&self) -> bool {
        return self
            .inner
            .playback
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| p.is_playing)
            .unwrap_or_default();
    }

    pub fn handle_action(&mut self, action: Action, sender: ContextSender) -> Result<(), Error> {
        let win = *self.inner.window.lock().unwrap();
        let modal = *self.inner.modal.lock().unwrap();

        match modal {
            Some(modal) => match modal {
                Modal::Devices => match action {
                    Action::Up => {
                        self.inner.devices.lock().unwrap().prev();
                    }
                    Action::Down => {
                        self.inner.devices.lock().unwrap().next();
                    }
                    Action::Select => {
                        sender.send_action(Action::Close)?;
                        let devices = self.inner.devices.lock().unwrap();

                        let play = devices.play;
                        if let Some(Device { id: Some(id), .. }) = devices.select() {
                            sender.send_action(Action::SetDevice { id, play })?
                        }
                    }
                    _ => {}
                },
            },
            None => match win {
                // _ => {}
                Window::Library => {
                    self.inner
                        .library
                        .lock()
                        .unwrap()
                        .handle_action(action, &self.spotify, &self.inner, sender.clone())?;
                }
            },
        }

        Ok(())
    }
}

impl Widget for &mut State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.inner.render(area, buf);
    }
}

#[derive(Default, Debug, Clone)]
pub struct InnerState {
    pub window: Arc<Mutex<Window>>,
    pub modal: Arc<Mutex<Option<Modal>>>,

    pub playback: Arc<Mutex<Option<Playback>>>,

    pub library: Arc<Mutex<LibraryState>>,

    pub devices: Arc<Mutex<DeviceState>>,
}

impl InnerState {
    pub fn init(&self, spotify: &AuthCodePkceSpotify, ctx: ContextSender) -> Result<(), Error> {
        self.fetch_playback(spotify, &ctx);
        self.fetch_featured(spotify, &ctx);
        self.fetch_library(spotify, &ctx);

        Ok(())
    }

    pub fn fetch_library(&self, spotify: &AuthCodePkceSpotify, ctx: &ContextSender) {
        let spotify = spotify.clone();
        let library = self.library.clone();
        let c = ctx.clone();
        tokio::spawn(async move {
            library.lock().unwrap().playlists.load();
            match spotify.current_user_playlists_manual(Some(20), None).await {
                Ok(playlists) => {
                    library.lock().unwrap().playlists.replace(playlists.convert_page())
                },
                Err(err) => {
                    c.send_error(err.into()).unwrap();
                    library.lock().unwrap().playlists.take()
                }
            };

            library.lock().unwrap().artists.load();
            match spotify.current_user_followed_artists(None, Some(20)).await {
                Ok(artists) => library.lock().unwrap().artists.replace(artists.convert_page()),
                Err(err) => {
                    c.send_error(err.into()).unwrap();
                    library.lock().unwrap().artists.take()
                }
            };

            library.lock().unwrap().albums.load();
            match spotify
                .current_user_saved_albums_manual(None, Some(20), None)
                .await
            {
                Ok(albums) => library.lock().unwrap().albums.replace(albums.convert_page()),
                Err(err) => {
                    c.send_error(err.into()).unwrap();
                    library.lock().unwrap().albums.take()
                }
            };

            library.lock().unwrap().shows.load();
            match spotify.get_saved_show_manual(Some(20), None).await {
                Ok(shows) => library.lock().unwrap().shows.replace(shows.convert_page()),
                Err(err) => {
                    c.send_error(err.into()).unwrap();
                    library.lock().unwrap().shows.take()
                }
            };
        });
    }

    pub fn fetch_playback(&self, spotify: &AuthCodePkceSpotify, ctx: &ContextSender) {
        let spot = spotify.clone();
        let playback = self.playback.clone();
        tokio::spawn(async move {
            let ctx = spot
                .current_playback(
                    None,
                    Some(vec![&AdditionalType::Track, &AdditionalType::Episode]),
                )
                .await
                .unwrap()
                .map(Playback::from);
            *playback.lock().unwrap() = ctx;
        });
    }

    pub fn fetch_featured(&self, spotify: &AuthCodePkceSpotify, ctx: &ContextSender) {
        let spot = spotify.clone();
        let lib = self.library.clone();
        tokio::spawn(async move {
            if let Ok(Some(release_radar)) = api::release_radar(&spot).await {
                lib.lock().unwrap().featured.push(release_radar.into());
            }

            if let Ok(Some(discover_weekly)) = api::discover_weekly(&spot).await {
                lib.lock().unwrap().featured.push(discover_weekly.into());
            }

            if let Ok(dm) = api::daily_mixes(&spot).await {
                lib.lock().unwrap().featured.extend(dm.into_iter().map(|v| v.into()));
            }
        });
    }
}

impl Widget for &mut InnerState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {

        let main = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(area);

        let win = *self.window.lock().unwrap();
        Clear.render(main[0], buf);
        StatefulWidget::render(win, main[0], buf, self);

        let modal = *self.modal.lock().unwrap();
        if let Some(modal) = modal {
            StatefulWidget::render(modal, main[0], buf, self);
        }

        match self.playback.lock().unwrap().as_ref() {
            Some(ctx) => ctx.render(main[1], buf),
            None => {
                Paragraph::new(vec![
                    Line::default(),
                    Line::from("<No Playback>").centered(),
                ])
                .render(main[1], buf);
            }
        }
    }
}
