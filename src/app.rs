use std::{collections::HashMap, io::stderr};

use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Widget,
    },
    Terminal,
};
use tokio::sync::mpsc;

use tupy::{
    api::{
        flow::{AuthFlow, Credentials, Pkce},
        response::{Playback, PlaybackItem},
        scopes, OAuth, Spotify, UserApi,
    },
    Duration, Error,
};

use crate::{
    errors::install_hooks, spotify_util::listen_for_authentication_code, tui, Locked, Shared,
};

static FPS: u8 = 20;

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Focus,
    Unfocus,
    Tick,
    Quit,
    None,

    Mouse(MouseEvent),

    Increment,
    Decrement,
    Toggle,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Countdown<const N: usize> {
    count: usize,
}
impl<const N: usize> Countdown<N> {
    pub fn decrement(&mut self) {
        self.count = self.count.saturating_sub(1);
    }
    pub fn is_ready(&self) -> bool {
        self.count == 0
    }
    pub fn reset(&mut self) {
        self.count = N;
    }
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub counter: u8,

    pub playback_poll: Countdown<100>,
    pub playback: Shared<Locked<Option<Playback>>>,
}

#[derive(Debug)]
pub struct App {
    pub terminal: tui::Tui,
    pub quit: bool,

    pub spotify: Spotify<Pkce>,
    pub state: State,
}

impl App {
    pub async fn refresh_token(&mut self) -> Result<()> {
        if self.spotify.api.token().is_expired() {
            self.spotify.api.refresh().await?;
        }
        Ok(())
    }

    pub async fn new() -> Result<Self> {
        let oauth = OAuth::from_env([scopes::USER_READ_PLAYBACK_STATE])
            .expect("Failed to get TUPY_CLIENT_ID and TUPY_REDIRECT environment variables.");

        let spotify =
            Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "rataify").unwrap();

        let app = Self {
            terminal: Terminal::new(CrosstermBackend::new(stderr())).unwrap(),
            quit: false,
            spotify,
            state: State::default(),
        };

        match app.spotify.api.refresh().await {
            Err(Error::TokenRefresh {
                redirect, state, ..
            }) => {
                let auth_url = app.spotify.api.authorization_url(true)?;
                let auth_code =
                    listen_for_authentication_code(&redirect, &auth_url, &state).await?;
                app.spotify.api.request_access_token(&auth_code).await?;
            }
            Err(e) => return Err(e.into()),
            _ => (),
        }

        *app.state.playback.lock().unwrap() = app.spotify.api.playback_state(None).await?;

        Ok(app)
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            f.render_widget(self.state.clone(), f.size());
        })?;
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Increment => self.state.counter = self.state.counter.saturating_add(1),
            Action::Decrement => self.state.counter = self.state.counter.saturating_sub(1),
            Action::Quit => self.quit = true,
            Action::Tick => {
                self.render()?;

                self.state.playback_poll.decrement();
                if self.state.playback_poll.is_ready() {
                    let playback = self.state.playback.clone();
                    let api = self.spotify.api.clone();
                    tokio::task::spawn(async move {
                        // TODO: Log output
                        api.refresh().await.unwrap();
                        *playback.lock().unwrap() = api.playback_state(None).await.unwrap();
                    });
                    self.state.playback_poll.reset();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_events(&self, keymaps: HashMap<KeyEvent, Action>, tx: mpsc::UnboundedSender<Action>) {
        let tick_rate = std::time::Duration::from_millis((1.0 / FPS as f32 * 1000.0) as u64);

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          crossterm::event::Event::Key(key) => {
                            if key.kind == crossterm::event::KeyEventKind::Press {
                              if let Some(action) = keymaps.get(&key) {
                                tx.send(*action).unwrap();
                              }
                            }
                          },
                          crossterm::event::Event::FocusGained => {
                            tx.send(Action::Focus).unwrap();
                          },
                          crossterm::event::Event::FocusLost => {
                            tx.send(Action::Unfocus).unwrap();
                          },
                          crossterm::event::Event::Mouse(mouse) => {
                            tx.send(Action::Mouse(mouse)).unwrap();
                          }
                          _ => {},
                        }
                      }
                      Some(Err(_)) => {
                        tx.send(Action::None).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = delay => {
                      tx.send(Action::Tick).unwrap();
                  },
                }
            }
        });
    }

    // Main Application Loop
    pub async fn run(&mut self, keymaps: HashMap<KeyEvent, Action>) -> Result<()> {
        install_hooks()?;
        tui::init()?;
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        self.handle_events(keymaps, action_tx.clone());

        while !self.quit {
            // application update
            if let Some(action) = action_rx.recv().await {
                self.update(action).await?;
            }
        }

        tui::restore()?;
        Ok(())
    }
}

impl Widget for State {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(5)])
            .split(area);

        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block.clone())
            .render(layout[0], buf);

        let block = Block::default()
            .borders(Borders::all());

        let playing = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(block.inner(layout[1]));

        {
            let playback = self.playback.lock().unwrap().clone();
            match playback.as_ref() {
                Some(playback) => match &playback.item {
                    PlaybackItem::Track(track) => {
                        let album = track.album.name.clone();
                        let artists = track
                            .artists
                            .iter()
                            .map(|a| a.name.as_str())
                            .collect::<Vec<&str>>()
                            .join(", ");

                        let scale =
                            layout[1].width as f32 / track.duration.num_milliseconds() as f32;
                        let progress = (playback .progress
                            .unwrap_or(Duration::zero())
                            .num_milliseconds() as f32
                            * scale) as u16;

                        let time = format!(
                            "{}/{}",
                            format_duration(playback.progress.unwrap_or(Duration::zero())),
                            format_duration(track.duration)
                        );
                        let tl = time.chars().count() as u16;
                        let title = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Length(playing[0].width - tl), Constraint::Length(tl)])
                            .split(playing[0]);

                        Span::from(track.name.clone())
                            .render(title[0], buf);
                        Span::from(time)
                            .render(title[1], buf);

                        Line::from(format!("{album} @ {artists}"))
                            .render(playing[1], buf);
                        Line::from(vec![
                            (0..progress)
                                .map(|_| "─")
                                .collect::<String>()
                                .green()
                                .bold(),
                            (0..(layout[1].width - progress))
                                .map(|_| "┄")
                                .collect::<String>()
                                .black(),
                        ])
                            .render(playing[2], buf);
                    }
                    PlaybackItem::Episode(episode) => {
                        let context = if let Some(show) = &episode.show {
                            show.publisher.as_deref().unwrap_or("").to_string()
                        } else {
                            String::new()
                        };

                        let scale =
                            layout[1].width as f32 / episode.duration.num_milliseconds() as f32;
                        let progress = (playback
                            .progress
                            .unwrap_or(Duration::zero())
                            .num_milliseconds() as f32
                            * scale) as u16;

                        let time = format!(
                            "{}/{}",
                            format_duration(playback.progress.unwrap_or(Duration::zero())),
                            format_duration(episode.duration)
                        );
                        let tl = time.chars().count() as u16;
                        let title = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Length(playing[0].width - tl), Constraint::Length(tl)])
                            .split(playing[0]);

                        Span::from(episode.name.clone())
                            .render(title[0], buf);
                        Span::from(time)
                            .render(title[1], buf);

                        Line::from(context)
                            .render(playing[1], buf);
                        Line::from(vec![
                            (0..progress)
                                .map(|_| "─")
                                .collect::<String>()
                                .green()
                                .bold(),
                            (0..(layout[1].width - progress))
                                .map(|_| "┄")
                                .collect::<String>()
                                .black(),
                        ])
                            .render(playing[2], buf);
                    }
                    PlaybackItem::Ad => {
                        Line::from("<Advertisement>")
                            .yellow()
                            .centered()
                            .render(playing[0], buf);
                    }
                    PlaybackItem::Unkown => {
                        Line::from("<Unknown Playback>")
                            .gray()
                            .centered()
                            .render(playing[0], buf);
                    }
                },
                None => {
                    Line::from("<No Playback>")
                        .red()
                        .centered()
                        .render(playing[0], buf);
                }
            }
        };
    }
}

fn format_duration(duration: Duration) -> String {
    if duration >= Duration::hours(1) {
        format!(
            "{:0>2}:{:0>2}:{:0>2}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    } else {
        format!(
            "{:0>2}:{:0>2}",
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    }
}
