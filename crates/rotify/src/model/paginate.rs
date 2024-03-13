use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Deserializer};
use crate::auth::OAuth;
use crate::Error;

pub fn parse_pagination<'de, D>(d: D) -> Result<Option<Paginate>, D::Error>
    where
        D: Deserializer<'de>,
{
    let url = Option::<String>::deserialize(d)?;
    match url {
        Some(url) => {
            let parts = url.split("?").collect::<Vec<&str>>();
            if parts.len() < 2 {
                return Ok(Some(Paginate::default()));
            }

            let paginate: Paginate = serde_qs::from_str(parts[1])
                .map_err(serde::de::Error::custom)?;

            Ok(Some(paginate))
        }
        None => Ok(None)
    }
}

#[derive(Default, Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct Paginate {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

impl Paginate {
    pub fn new(offset: Option<usize>, limit: Option<usize>) -> Self {
        Self {
            offset,
            limit,
        }
    }
}

pub struct PaginateCursor {
    pub next: Option<Paginate>,
    pub prev: Option<Paginate>,
}

pub struct PaginationIter<T, S>
where
    S: Clone,
    T: Send
{
    oauth: Arc<Mutex<OAuth>>,
    state: Option<S>,
    cursor: PaginateCursor,
    call: Box<dyn Fn(Paginate, String, Option<S>) -> Pin<Box<dyn Future<Output = (PaginateCursor, Result<T, Error>)> + Send>>>
}

impl<T, S> PaginationIter<T, S>
where
    S: Clone,
    T: Send
{
    pub fn new<U>(oauth: Arc<Mutex<OAuth>>, state: Option<S>, next: Paginate, call: U) -> Self
    where
        U: Fn(Paginate, String, Option<S>) -> Pin<Box<dyn Future<Output = (PaginateCursor, Result<T, Error>)> + Send>> + 'static,
    {
        Self {
            oauth,
            state,
            cursor: PaginateCursor {
                next: Some(next),
                prev: None,
            },
            call: Box::new(call),
        }
    }

    pub async fn next(&mut self) -> Option<Result<T, Error>> {
        match self.oauth.lock().unwrap().update().await {
            Ok(token) => {
                if let Some(next) = self.cursor.next {
                    let (cursor, result) = (self.call)(
                        next,
                        token.unwrap().to_header(),
                        self.state.clone()
                    ).await;
                    self.cursor = cursor;
                    Some(result)
                } else {
                    None
                }
            },
            Err(err) => {
                Some(Err(err.into()))
            }
        }
    }

    pub async fn previous(&mut self) -> Option<Result<T, Error>> {
        match self.oauth.lock().unwrap().update().await {
            Ok(token) => {
                if let Some(previous) = self.cursor.prev {
                    let (cursor, result) = (self.call)(
                        previous,
                        token.unwrap().to_header(),
                        self.state.clone()
                    ).await;
                    self.cursor = cursor;
                    Some(result)
                } else {
                    None
                }
            },
            Err(err) => {
                Some(Err(err.into()))
            }
        }
    }
}

/// Two-way async iterator. Mainly for use with paginated endpoints. Allows for the next and previous
/// urls to be followed automatically.
pub trait AsyncIter {
    type Item;

    fn next(&mut self) -> impl Future<Output=Option<Self::Item>>;
    fn prev(&mut self) -> impl Future<Output=Option<Self::Item>>;
}
