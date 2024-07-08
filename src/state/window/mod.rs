use color_eyre::Result;

pub mod library;
pub mod queue;
pub mod landing;

use std::fmt::Debug;
use serde::Deserialize;

use tupy::{api::{flow::Pkce, response::{Paginated, Paged}}, Pagination};
use tokio::sync::Mutex;

use super::Loading;
use crate::{Shared, Locked, PAGE_SIZE};

#[derive(Debug, Clone)]
pub struct WindowState {
    pub library: Shared<Locked<library::LibraryState>>,
    pub queue: Shared<Locked<queue::QueueState>>,
    pub landing: Shared<Locked<landing::Landing>>
}

impl WindowState {
    pub async fn new(dir: &str, api: &Pkce) -> Result<Self> {
        Ok(Self {
            library: Shared::new(Locked::new(library::LibraryState::new(dir, api).await?)),
            queue: Shared::default(),
            landing: Shared::default()
        })
    }
}

#[derive(Debug, Clone)]
pub struct Pages<R, P>
    where 
        R: Clone + Debug + Send,
        P: Clone + Debug + Send,
{
    pub pager: Shared<Mutex<Paginated<R, P, Pkce, PAGE_SIZE>>>,
    pub items: Shared<Locked<Option<Loading<R>>>>,
    pub pages: Shared<Locked<(usize, usize)>>,
}

impl<R, P> Pages<R, P>
    where 
        R: Clone + Debug + Send + Paged + 'static,
        P: Clone + Debug + Send + Deserialize<'static> + 'static,
{
    pub fn new(pager: Paginated<R, P, Pkce, PAGE_SIZE>) -> Self {
        Self {
            pager: Shared::new(Mutex::new(pager)),
            items: Shared::default(),
            pages: Shared::default(),
        }
    }

    pub async fn has_next(&self) -> bool {
        self.pager.lock().await.has_next()
    }

    pub async fn has_prev(&self) -> bool {
        self.pager.lock().await.has_prev()
    }

    pub async fn next(&mut self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let pages = self.pages.clone();
        tokio::task::spawn(async move {
            let mut pager = pager.lock().await;
            *items.lock().unwrap() = Some(Loading::from(pager.next().await.unwrap()));
            *pages.lock().unwrap() = (pager.page(), pager.total_pages())
        });
        Ok(())
    }

    pub async fn prev(&mut self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let pages = self.pages.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            *items.lock().unwrap() = Some(Loading::from(pager.prev().await.unwrap()));
            *pages.lock().unwrap() = (pager.page(), pager.total_pages())
        });
        Ok(())
    }
}
