use std::fmt::{Debug, Display, Formatter};

use color_eyre::{Report, Section};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Custom(Report),
    Response(rotify::Error),
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    NoDevice,
}

impl Error {
    pub fn custom<S: Display + Debug + Send + Sync + 'static>(message: S) -> Self {
        Self::Custom(Report::msg(message.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", match self {
            Self::Custom(msg) => msg.to_string(),
            Self::NoDevice => "No device found".to_string(),
            Self::Response(err) => Report::from(err).to_string(),
            Self::Io(err) => err.to_string(),
            Self::Reqwest(err) => err.to_string(),
        })
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<Error> for Report {
    fn from(error: Error) -> Self {
        match error {
            Error::Custom(report) => report,
            Error::Reqwest(err) => Report::from(err),
            Error::Io(err) => Report::from(err),
            Error::Response(err) => Report::from(err),
            Error::NoDevice => Report::msg("No device found")
                .suggestion("Try again later")
                .suggestion("Make sure the device is active")
                .suggestion("For mobile devices, this means they have to be unlocked or playing"),
        }
    }
}

impl From<Report> for Error {
    fn from(error: Report) -> Self {
        Error::Custom(error)
    }
}
