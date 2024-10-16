use std::{io::stdout, ops::{Deref, DerefMut}};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::Backend, Terminal};

use crate::{event::EventHandler, Error};

pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B> Deref for Tui<B>
where
    B: Backend
{
    type Target = Terminal<B>;

    fn deref(&self) -> &Self::Target {
        &self.terminal    
    }
}

impl<B> DerefMut for Tui<B>
where
    B: Backend
{
    fn deref_mut(&mut self) -> &mut Self::Target {
       &mut self.terminal 
    }
}

impl<B> Tui<B>
where
    B: Backend,
{
    pub fn new(backend: B, tick_rate: u64, render_rate: u64) -> Result<Self, Error> {
        Ok(Self {
            terminal: Terminal::new(backend)?,
            events: EventHandler::new(tick_rate, render_rate),
        })
    }

    pub fn init(&mut self) -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture,)?;

        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        Ok(())
    }

    pub fn reset() -> Result<(), Error> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Error> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
