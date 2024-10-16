mod playback;
pub mod window;

use std::{
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
    widgets::{Paragraph, StatefulWidget, Widget},
};
use rspotify::{clients::OAuthClient, model::SimplifiedPlaylist, scopes, AuthCodePkceSpotify, Credentials, OAuth};
use window::{library::LibraryState, Window};

use crate::{api, Error};

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
    spotify: AuthCodePkceSpotify,
    inner: InnerState,
}

impl State {
    pub async fn new(inner: InnerState) -> Result<Self, Error> {
        let mut spotify = AuthCodePkceSpotify::with_config(
            Credentials::from_env().ok_or(Error::custom(
                "failed to parse spotify Credentials from environment variables",
            ))?,
            OAuth::from_env(scopes!["user-read-playback-state"]).ok_or(Error::custom(
                "failed to parse spotify OAuth from environment variables",
            ))?,
            rspotify::Config {
                cache_path: dirs::cache_dir()
                    .unwrap()
                    .join("rataify")
                    .join("token.json"),
                token_cached: true,
                token_refreshing: true,
                ..Default::default()
            },
        );

        let url = spotify.get_authorize_url(None)?;
        spotify.prompt_for_token(url.as_str()).await?;

        inner.init(spotify.clone())?;

        Ok(Self {
            playback_ping: Timer::new(
                Duration::from_secs(5),
                |spotify: AuthCodePkceSpotify, inner: InnerState| async move {
                    let playback = inner.playback.clone();
                    tokio::spawn(async move {
                        let ctx = spotify
                            .current_playback(None, None::<Vec<_>>)
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
    window: Window,

    playback: Arc<Mutex<Option<Playback>>>,

    library: Arc<Mutex<LibraryState>>,

    daily_mixes: Arc<Mutex<Vec<SimplifiedPlaylist>>>,
    release_radar: Arc<Mutex<Option<SimplifiedPlaylist>>>,
    discover_weekly: Arc<Mutex<Option<SimplifiedPlaylist>>>,
}

impl InnerState {
    pub fn init(&self, spotify: AuthCodePkceSpotify) -> Result<(), Error> {
        let spot = spotify.clone();
        let playback = self.playback.clone();
        tokio::spawn(async move {
            let ctx = spot
                .current_playback(None, None::<Vec<_>>)
                .await
                .unwrap()
                .map(Playback::from);
            *playback.lock().unwrap() = ctx;
        });


        let spot = spotify.clone();
        let mixes = self.daily_mixes.clone();
        let rr = self.release_radar.clone();
        let dw = self.discover_weekly.clone();
        tokio::spawn(async move {
            if let Ok((release, discover)) = api::release_discover(&spot).await {
                *rr.lock().unwrap() = release;
                *dw.lock().unwrap() = discover;
            }

            if let Ok(dm) = api::daily_mixes(&spot).await {
                *mixes.lock().unwrap() = dm;
            }
        });

        Ok(())
    }
}

impl Widget for &mut InnerState {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let main = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(area);

        StatefulWidget::render(self.window, main[0], buf, self);

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
