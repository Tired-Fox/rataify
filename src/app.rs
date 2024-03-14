use std::collections::HashMap;

use color_eyre::Report;
use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::action;
use crate::action::{Action, Private, Public};
use crate::config::Config;
use crate::error::Error;
use crate::event::{Event, Tui};
use rotify::model::player::Repeat;
use rotify::{Spotify, SpotifyRequest};
use crate::state::{MainWindow, ModalWindow, Move, State, TABS, WindowState};

pub struct App {
    /// App should quit on next event loop
    should_quit: bool,
    ui: Option<Box<dyn FnMut(&mut State, &mut Frame) + 'static>>,
    /// Actions Output Channel
    pub actions: tokio::sync::mpsc::UnboundedSender<Action>,

    /// Actions Input Channel
    input: tokio::sync::mpsc::UnboundedReceiver<Action>,

    pub spotify: Spotify,

    /// State
    state: State,
}

impl App {
    /// Async app setup to also initialize the spotify api interactions.
    ///
    /// The interactions require an access token so http requests may run on init.
    pub async fn new() -> color_eyre::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let actions = tx.clone();

        // Fetch playback on interval asynchronously, so it doesn't block render and event loop
        tokio::task::spawn(async move {
            loop {
                // {
                //     let playback = spot.playback().await.ok();
                //     let state = &mut data;
                //     state.playback.now_playing(playback);
                // }
                actions
                    .send(Action::Private(Private::FetchPlayback))
                    .unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        Ok(Self {
            should_quit: false,
            ui: None,

            input: rx,

            spotify: Spotify::new()?,
            state: State::new().await,

            actions: tx,
        })
    }

    pub fn with_ui<F>(mut self, ui: F) -> Self
        where
            F: FnMut(&mut State, &mut Frame) + 'static + Clone,
    {
        self.ui = Some(Box::new(ui));
        self
    }

    async fn tab_update(&mut self) {
        match self.state.window.main {
            MainWindow::Queue => {
                if self.state.queue.unset() {
                    if let Ok(mut queue) = self.spotify.player().get_queue().send().await {
                        // TODO: Add additional state for if the track is liked
                        // if let Ok(liked) = self.spotify
                        //     .tracks()
                        //     .check_saved_tracks(queue.queue.iter().map(|i| i.id()))
                        //     .send()
                        //     .await {
                        //     queue.queue.iter_mut().enumerate().for_each(|(i, q)| q.set_liked(*liked.get(i).unwrap()))
                        // }
                        self.state.queue.set_queue(Some(queue));
                    }
                }
            }
            _ => {}
        }
    }

    async fn fetch_state() {}

    async fn update(&mut self, action: Action) -> color_eyre::Result<()> {
        match action {
            Action::Public(public) => match public {
                Public::Next => self.spotify.player().skip_to_next().send().await?,
                Public::Previous => self.spotify.player().skip_to_previous().send().await?,
                Public::Down => self.state.move_with(Move::Down),
                Public::Up => self.state.move_with(Move::Up),
                Public::Left => self.state.move_with(Move::Left),
                Public::Right => self.state.move_with(Move::Right),
                Public::Select => {
                    match self.state.window_state {
                        WindowState::Modal => {
                            match self.state.window.modal {
                                ModalWindow::DeviceSelect => {
                                    let play = match self.state.device_select.end_action {
                                        Some(Action::Public(Public::Play)) => Some(true),
                                        Some(Action::Public(Public::Pause)) => Some(false),
                                        _ => None,
                                    };

                                    let device = self.state.device_select.device();
                                    if let Some(device) = device.as_ref() {
                                        let mut transfer = self.spotify
                                            .player()
                                            .transfer_playback([device.id.clone()]);
                                        if let Some(play) = play {
                                            transfer = transfer.play(play);
                                        }

                                        let response = transfer.send().await;

                                        self.state.device_select.reset();
                                        self.state.back();

                                        response?;
                                    }

                                    self.state.device_select.reset();
                                    self.state.back();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Public::Help => self.state.show_modal(ModalWindow::Help),
                Public::Play => self.spotify.player().play().send().await?,
                Public::ToggleShuffle => self.spotify.player().shuffle(!self.state.playback.shuffle()).send().await?,
                Public::ToggleRepeat => self.spotify.player().repeat(match self.state.playback.repeat() {
                    Repeat::Context => Repeat::Off,
                    Repeat::Off => Repeat::Track,
                    Repeat::Track => Repeat::Context,
                }).send().await?,
                Public::Pause => self.spotify.player().pause().send().await?,
                Public::TogglePlayback => match self.state.playback.playing() {
                    true => self.spotify.player().pause().send().await?,
                    false => self.spotify.player().play().send().await?,
                },
                Public::Back => {
                    if self.state.back() {
                        self.should_quit = true;
                    }
                }
                Public::Exit => self.should_quit = true,
                Public::SelectDevice => {
                    // TODO: Handle what happens when there are no devices
                    //  probably show an error message that a device needs to be started
                    let devices = self.spotify.player().get_devices().send().await?;
                    self.state.device_select.set_devices(devices);

                    let device = self
                        .state
                        .playback
                        .current
                        .as_ref()
                        .map(|p| p.device.clone());
                    self.state.device_select.select(device);
                    self.state.show_modal(ModalWindow::DeviceSelect);
                }
                Public::NextTab => match TABS.iter().position(|t| t == &self.state.window.main) {
                    Some(index) => {
                        self.state.window.main = *TABS.get((index + 1) % TABS.len()).unwrap();
                        self.tab_update().await;
                    }
                    _ => {
                        self.state.window.main = **(TABS.first().as_ref().unwrap());
                        self.tab_update().await;
                    }
                },
                Public::PreviousTab => match TABS.iter().position(|t| t == &self.state.window.main)
                {
                    Some(mut index) => {
                        let ni = (index as isize) - 1;
                        index = if ni < 0 {
                            (TABS.len() as isize + ni) as usize
                        } else {
                            ni as usize
                        };
                        self.state.window.main = *TABS.get(index).unwrap();
                        self.tab_update().await;
                    }
                    _ => {
                        self.state.window.main = **(TABS.last().as_ref().unwrap());
                        self.tab_update().await;
                    }
                },
                _ => unimplemented!(),
            },
            Action::Private(private) => match private {
                Private::Tick => {}
                Private::Focus => self.state.focused = true,
                Private::Unfocus => self.state.focused = false,
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn get_action(&mut self, event: Event, keymap: &HashMap<KeyEvent, Action>) -> Action {
        match event {
            Event::Quit => Action::from(Public::Exit),
            Event::Error => Action::None,
            Event::Tick => Action::from(Private::Tick),
            Event::Render => Action::from(Private::Render),
            Event::FocusGained => Action::from(Private::Focus),
            Event::FocusLost => Action::from(Private::Unfocus),
            Event::Key(key) => {
                if keymap.contains_key(&key) {
                    return *keymap.get(&key).unwrap();
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    pub async fn run(&mut self, config: Config) -> color_eyre::Result<()> {
        let keymaps = &config.keymaps;

        let mut terminal = Tui::new()?.title("Rataify");
        terminal.enter()?;

        loop {
            let event = terminal.events.next().await?;

            let action = self.get_action(event, keymaps);
            self.actions.send(action.clone())?;

            let mut fetched_playback = false;
            while let Ok(action) = self.input.try_recv() {
                if let Action::Private(Private::Render) = action {
                    if let Some(ui) = &mut self.ui {
                        terminal
                            .draw(|frame: &mut Frame| ui(&mut self.state, frame))
                            .unwrap();
                    }
                } else if let Action::Private(Private::FetchPlayback) = action {
                    if !fetched_playback {
                        if let Ok(playback) = self.spotify.player().playback().send().await {
                            // TODO: Add additional state for if it is liked
                            // if let Some(playback) = &mut playback {
                            //     if let Ok(liked) = self.spotify.tracks().check_saved_tracks(vec![playback.item.as_ref().unwrap().id()]).await {
                            //         playback.item.as_mut().unwrap().set_liked(*liked.first().unwrap());
                            //     }
                            // }
                            self.state.playback.now_playing(playback);
                        }
                    }
                    fetched_playback = true;
                } else {
                    self.update(action).await?;
                }
            }

            if self.should_quit {
                break;
            }
        }

        terminal.exit()?;
        Ok(())
    }
}
