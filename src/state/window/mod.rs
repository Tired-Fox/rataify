use color_eyre::Result;

pub mod library;
pub mod queue;

use tupy::api::flow::Pkce;

use crate::{Shared, Locked};

#[derive(Debug, Clone)]
pub struct WindowState {
    pub library: Shared<Locked<library::LibraryState>>,
    pub queue: Shared<Locked<queue::QueueState>>,
}

impl WindowState {
    pub async fn new(dir: &str, api: &Pkce) -> Result<Self> {
        Ok(Self {
            library: Shared::new(Locked::new(library::LibraryState::new(dir, api).await?)),
            queue: Shared::default()
        })
    }
}
