use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use crate::auth::OAuth;
use crate::{Error, SpotifyRequest, SpotifyResponse};
use crate::model::user::{TopItems, UserProfile, UserPublicProfile};

pub struct CurrentUserProfileBuilder<'a> {
    oauth: &'a mut OAuth,
}

impl <'a> CurrentUserProfileBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self { oauth }
    }
}

impl<'a> SpotifyRequest<UserProfile> for CurrentUserProfileBuilder<'a> {
    async fn send(self) -> Result<UserProfile, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetUserProfileBuilder<'a> {
    oauth: &'a mut OAuth,
    user_id: String
}

impl<'a> GetUserProfileBuilder<'a> {
    pub fn new<S: Into<String>>(oauth: &'a mut OAuth, user_id: S) -> Self {
        Self { oauth, user_id: user_id.into() }
    }
}

impl<'a> SpotifyRequest<UserPublicProfile> for GetUserProfileBuilder<'a> {
    async fn send(self) -> Result<UserPublicProfile, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/users/{}", self.user_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

//
// #[derive(Debug, Serialize, Clone, Copy)]
// #[serde(rename_all = "snake_case")]
// pub enum TimeRange {
//     ShortTerm,
//     MediumTerm,
//     LongTerm
// }
//
// impl Display for TimeRange {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", match self {
//             TimeRange::ShortTerm => "short_term",
//             TimeRange::MediumTerm => "medium_term",
//             TimeRange::LongTerm => "long_term",
//         })
//     }
// }
//
// pub struct UsersTopItemsBuilder<'a, I> {
//     oauth: &'a mut OAuth,
//     item_type: UserItemType,
//     time_range: Option<TimeRange>,
//     limit: Option<u32>,
//     offset: Option<u32>,
//     _marker: PhantomData<I>
// }
//
// impl<'a, I> UsersTopItemsBuilder<'a, I> {
//     pub fn new(oauth: &'a mut OAuth, item_type: UserItemType) -> Self {
//         Self {
//             oauth,
//             item_type,
//             time_range: None,
//             limit: None,
//             offset: None,
//             _marker: PhantomData
//         }
//     }
//
//     pub fn time_range(mut self, time_range: TimeRange) -> Self {
//         self.time_range = Some(time_range);
//         self
//     }
//
//     pub fn limit(mut self, limit: u32) -> Self {
//         self.limit = Some(limit);
//         self
//     }
//
//     pub fn offset(mut self, offset: u32) -> Self {
//         self.offset = Some(offset);
//         self
//     }
// }
//
// impl<'a, I> SpotifyRequest<TopItems<I>> for UsersTopItemsBuilder<'a, I>
// where
//     I: Deserialize<'static> + Clone + Debug + PartialEq
// {
//     async fn send(self) -> Result<TopItems<I>, Error> {
//         self.oauth.update().await?;
//
//         let mut query = HashMap::new();
//         if let Some(time_range) = self.time_range {
//             query.insert("time_range", time_range.to_string());
//         }
//         if let Some(limit) = self.limit {
//             query.insert("limit", limit.to_string());
//         }
//         if let Some(offset) = self.offset {
//             query.insert("offset", offset.to_string());
//         }
//
//         reqwest::Client::new()
//             .get(format!("https://api.spotify.com/v1/me/top/{}", self.item_type))
//             .header("Authorization", self.oauth.token().unwrap().to_header())
//             .query(&query)
//             .send()
//             .to_spotify_response()
//             .await
//     }
// }
