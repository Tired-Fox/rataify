use color_eyre::Result;

pub mod library;
pub mod queue;
pub mod landing;

use std::future::Future;
use std::pin::Pin;
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

    pub async fn next(&self) -> Result<()> {
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

    pub async fn refresh(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let pages = self.pages.clone();
        tokio::task::spawn(async move {
            let mut pager = pager.lock().await;
            *items.lock().unwrap() = Some(Loading::from(pager.current().await.unwrap()));
            *pages.lock().unwrap() = (pager.page(), pager.total_pages())
        });
        Ok(())
    }

    pub async fn prev(&self) -> Result<()> {
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

#[derive(Clone)]
pub struct MappedPages<M, R, P>
    where 
        M: Clone + Send, 
        R: Clone + Send,
        P: Clone + Send,
{
    pub pager: Shared<Mutex<Paginated<R, P, Pkce, PAGE_SIZE>>>,
    pub pages: Shared<Locked<(usize, usize, usize)>>,

    pub mapper: Shared<dyn Fn(Option<R>, Pkce) -> Pin<Box<dyn Future<Output = Result<Option<M>>> + Send>> + Send + Sync>,
    pub items: Shared<Locked<Option<Loading<M>>>>,
}

impl<M, R, P> MappedPages<M, R, P>
    where 
        M: Clone + Send + 'static,
        R: Clone + Send + Paged + 'static,
        P: Clone + Send + Deserialize<'static> + 'static,
{
    pub fn new<F>(pager: Paginated<R, P, Pkce, PAGE_SIZE>, mapper: F) -> Self
    where
        F: Fn(Option<R>, Pkce) -> Pin<Box<dyn Future<Output = Result<Option<M>>> + Send>> + Send + Sync + 'static
    {
        Self {
            pager: Shared::new(Mutex::new(pager)),
            pages: Shared::default(),
            mapper: Shared::new(mapper),
            items: Shared::default(),
        }
    }

    pub async fn has_next(&self) -> bool {
        self.pager.lock().await.has_next()
    }

    pub async fn has_prev(&self) -> bool {
        self.pager.lock().await.has_prev()
    }

    pub async fn next(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let pages = self.pages.clone();
        let mapper = self.mapper.clone();
        tokio::task::spawn(async move {
            let mut pager = pager.lock().await;
            *items.lock().unwrap() = Some(Loading::from(mapper(pager.next().await.unwrap(), pager.flow().clone()).await.unwrap()));
            *pages.lock().unwrap() = (pager.page(), pager.total_pages(), pager.total())
        });
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let pages = self.pages.clone();
        let mapper = self.mapper.clone();
        tokio::task::spawn(async move {
            let mut pager = pager.lock().await;
            *items.lock().unwrap() = Some(Loading::from(mapper(pager.current().await.unwrap(), pager.flow().clone()).await.unwrap()));
            *pages.lock().unwrap() = (pager.page(), pager.total_pages(), pager.total())
        });
        Ok(())
    }

    pub async fn prev(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let pages = self.pages.clone();
        let mapper = self.mapper.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            *items.lock().unwrap() = Some(Loading::from(mapper(pager.prev().await.unwrap(), pager.flow().clone()).await.unwrap()));
            *pages.lock().unwrap() = (pager.page(), pager.total_pages(), pager.total())
        });
        Ok(())
    }
}
