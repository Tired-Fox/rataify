use std::{fmt::Display, io::Write, panic, path::PathBuf};

use color_eyre::{config::HookBuilder, eyre};

use crate::tui;

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            tui::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

lazy_static::lazy_static! {
    pub static ref ERROR_FILE_PATH: PathBuf = dirs::data_local_dir().unwrap().join("rataify").join("errors.log");
}

pub struct StdError;
impl StdError {
    pub fn clear_error_file() -> std::io::Result<()> {
        if ERROR_FILE_PATH.exists() {
            let _ = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(ERROR_FILE_PATH.as_path())?;
        }
        Ok(())
    }
}

pub trait LogError<T, E> {
    fn log_error_format_error<D: Display>(error: D) -> String {
        format!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), error)
    }

    fn log_error(self);
    fn log_error_or(self, or: T) -> T;
    fn log_error_ok(self) -> Option<T>;
}

pub trait LogErrorDefault<T: Default, E> {
    fn log_error_or_default(self) -> T;
}

impl<T: Default, E: std::error::Error> LogErrorDefault<T, StdError> for std::result::Result<T, E> {
    fn log_error_or_default(self) -> T {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            return match self {
                Ok(t) => t,
                Err(e) => {
                    let _ = writeln!(file, "{}", Self::log_error_format_error(e));
                    T::default()
                }
            }
        }
        self.unwrap_or_default()
    }
}

impl<T> LogError<T, color_eyre::Report> for std::result::Result<T, color_eyre::Report> {
    fn log_error_ok(self) -> Option<T> {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            return match self {
                Ok(t) => Some(t),
                Err(e) => {
                    let _ = writeln!(file, "{}", Self::log_error_format_error(e));
                    None
                }
            }
        }
        self.ok()
    }

    fn log_error(self) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            if let Err(e) = self {
                let _ = writeln!(file, "{}", Self::log_error_format_error(e));
            }
        }
    }

    fn log_error_or(self, or: T) -> T {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            return match self {
                Ok(t) => t,
                Err(e) => {
                    let _ = writeln!(file, "{}", Self::log_error_format_error(e));
                    or
                }
            }
        }
        or
    }
}

impl<T, E: std::error::Error> LogError<T, StdError> for std::result::Result<T, E> {
    fn log_error_ok(self) -> Option<T> {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            return match self {
                Ok(t) => Some(t),
                Err(e) => {
                    let _ = writeln!(file, "{}", Self::log_error_format_error(e));
                    None
                }
            }
        }
        self.ok()
    }

    fn log_error(self) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            if let Err(e) = self {
                let _ = writeln!(file, "{}", Self::log_error_format_error(e));
            }
        }
    }

    fn log_error_or(self, or: T) -> T {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            return match self {
                Ok(t) => t,
                Err(e) => {
                    let _ = writeln!(file, "{}", Self::log_error_format_error(e));
                    or
                }
            }
        }
        or
    }
}

impl<T: Default> LogErrorDefault<T, color_eyre::Report> for std::result::Result<T, color_eyre::Report> {
    fn log_error_or_default(self) -> T {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ERROR_FILE_PATH.as_path()) {
            return match self {
                Ok(t) => t,
                Err(e) => {
                    let _ = writeln!(file, "{}", Self::log_error_format_error(e));
                    T::default()
                }
            }
        }
        self.unwrap_or_default()
    }
}
