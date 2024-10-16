use rspotify::ClientError;
use tokio::sync::mpsc::error::TryRecvError;

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    RecvError(tokio::sync::mpsc::error::TryRecvError),
    Spotify(ClientError),
    Custom(String),
}

pub struct Error {
    kind: ErrorKind,
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
            ErrorKind::RecvError(recv) => match recv {
                TryRecvError::Empty => "failed to recieve event; stream is empty".to_string(),
                TryRecvError::Disconnected => "failed to recieve event; stream is disconnected".to_string(),
            }
            ErrorKind::Custom(message) => message.clone(),
            ErrorKind::Spotify(spotify) => spotify.to_string(),
        }
    }
}

impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self {
            kind: ErrorKind::Spotify(value)
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
            kind: ErrorKind::RecvError(value)
        }
    }
}
