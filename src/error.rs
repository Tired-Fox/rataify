use std::fmt::{Display, Formatter};
use color_eyre::{Report, Section};
pub use color_eyre::Result;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new<S: Display>(message: S) -> Self {
        Self { message: message.to_string() }
    }

    pub fn setup() -> Result<()> {
        color_eyre::install()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UNKOWN: {}", self.message)
    }
}

impl From<Error> for Report {
    fn from(error: Error) -> Self {
        Report::msg(error.to_string())
            .suggestion("Try again later")
    }
}
