use std::io::{self, stderr, Stderr};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stderr>>;

/// Initialize the terminal
pub fn init() -> io::Result<()> {
    execute!(stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    Ok(())
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}
