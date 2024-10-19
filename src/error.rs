use rspotify::{model::IdError, ClientError};
use tokio::sync::mpsc::error::TryRecvError;

#[derive(Debug)]
pub enum SpotifyErrorKind {
    ClientError(ClientError),
    IdError(IdError)
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Stream(StreamError),
    Spotify(SpotifyErrorKind),
    Custom(String),
    Group(Vec<Error>)
}

#[derive(Debug)]
pub enum StreamError {
    Empty,
    Closed,
    Disconnected,
}

pub struct Error {
    pub kind: ErrorKind,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[31mERROR\x1b[0m: {}", self.message())
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn custom(message: impl std::fmt::Display) -> Self {
        Self {
            kind: ErrorKind::Custom(message.to_string())
        }
    }

    pub fn message(&self) -> String {
        match &self.kind {
            ErrorKind::Io(io) => io.to_string(),
            ErrorKind::Stream(se) => match se {
                StreamError::Empty => "failed to recieve event; stream is empty".to_string(),
                StreamError::Disconnected => "failed to recieve event; stream disconnected".to_string(),
                StreamError::Closed => "failed to send event; stream is closed".to_string(),
            }
            ErrorKind::Custom(message) => message.clone(),
            ErrorKind::Spotify(spotify) => match spotify {
                SpotifyErrorKind::ClientError(client) => client.to_string(),
                SpotifyErrorKind::IdError(id) => id.to_string(),
            },
            ErrorKind::Group(group) => group.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join("\n")
        }
    }
}

impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self {
            kind: ErrorKind::Spotify(SpotifyErrorKind::ClientError(value))
        }
    }
}

impl From<IdError> for Error {
    fn from(value: IdError) -> Self {
        Self {
            kind: ErrorKind::Spotify(SpotifyErrorKind::IdError(value))
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error {
            kind: ErrorKind::Io(value)
        }
    }
}

impl From<tokio::sync::mpsc::error::TryRecvError> for Error {
    fn from(value: tokio::sync::mpsc::error::TryRecvError) -> Self {
        Error {
            kind: ErrorKind::Stream(match value {
                TryRecvError::Empty => StreamError::Empty,
                TryRecvError::Disconnected => StreamError::Disconnected,
            })
        }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(_value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Error {
            kind: ErrorKind::Stream(StreamError::Closed)
        }
    }
}
