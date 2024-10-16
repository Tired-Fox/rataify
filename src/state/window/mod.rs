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
use crate::{Shared, Locked, PAGE_SIZE, errors::LogErrorDefault};

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

#[derive(Clone, Debug, Default)]
pub struct Page {
    pub offset: usize,
    pub total: usize,
    pub limit: usize,
    pub page: usize,
    pub max_page: usize,
}

impl Page {
    pub fn from_paged<P: Paged>(paged: &P) -> Self {
        Self {
            offset: paged.offset(),
            total: paged.total(),
            limit: paged.limit(),
            page: paged.page(),
            max_page: paged.max_page()
        }
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
    pub page: Shared<Locked<Page>>,
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
            page: Shared::default(),
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
        let page = self.page.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            let next = pager.next().await.log_error_or_default();
            if let Some(n) = next.as_ref() {
                *page.lock().unwrap() = Page::from_paged(n);
            }
            *items.lock().unwrap() = Some(Loading::from(next));
        });
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let page = self.page.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            let current = pager.current().await.log_error_or_default();
            if let Some(c) = current.as_ref() {
                *page.lock().unwrap() = Page::from_paged(c);
            }
            *items.lock().unwrap() = Some(Loading::from(pager.current().await.log_error_or_default()));
        });
        Ok(())
    }

    pub async fn prev(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let page = self.page.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            let prev = pager.prev().await.log_error_or_default();
            if let Some(p) = prev.as_ref() {
                *page.lock().unwrap() = Page::from_paged(p);
            }
            *items.lock().unwrap() = Some(Loading::from(prev));
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
    pub page: Shared<Locked<Page>>,

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
            page: Shared::default(),
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
        let page = self.page.clone();
        let mapper = self.mapper.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            let next = pager.next().await.unwrap();
            if let Some(n) = next.as_ref() {
                *page.lock().unwrap() = Page::from_paged(n);
            }
            *items.lock().unwrap() = Some(Loading::from(mapper(next, pager.flow().clone()).await.log_error_or_default()));
        });
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let page = self.page.clone();
        let mapper = self.mapper.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            let current = pager.current().await.unwrap();
            if let Some(c) = current.as_ref() {
                *page.lock().unwrap() = Page::from_paged(c);
            }
            *items.lock().unwrap() = Some(Loading::from(mapper(current, pager.flow().clone()).await.log_error_or_default()));
        });
        Ok(())
    }

    pub async fn prev(&self) -> Result<()> {
        *self.items.lock().unwrap() = Some(Loading::Loading);

        let items = self.items.clone();
        let pager = self.pager.clone();
        let page = self.page.clone();
        let mapper = self.mapper.clone();
        tokio::spawn(async move {
            let mut pager = pager.lock().await;
            let prev = pager.prev().await.unwrap();
            if let Some(p) = prev.as_ref() {
                *page.lock().unwrap() = Page::from_paged(p);
            }
            *items.lock().unwrap() = Some(Loading::from(mapper(prev, pager.flow().clone()).await.log_error_or_default()));
        });
        Ok(())
    }
}
