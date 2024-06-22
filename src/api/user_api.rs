use std::{fmt::Debug, future::Future};

use serde::Deserialize;

use crate::{pinned, Error};

use super::{
    auth::Token,
    flow::AuthFlow,
    request::{self, TimeRange},
    response::{IntoUserTopItemType, Paginated, Profile, TopItems},
    scopes, validate_scope, DefaultResponse, SpotifyResponse, API_BASE_URL,
};

pub trait UserApi: AuthFlow {
    /// Get detailed profile information about the current user (including the current user's username).
    ///
    /// # Scopes
    /// - `user-read-private` [optional]
    ///     - Access to the `product`, `explicit_content`, and `country` fields
    /// - `user-read-email` [optional]
    ///     - Access to the `email` field
    fn get_current_user_profile(&self) -> impl Future<Output = Result<Profile, Error>> {
        async {
            // Get the token: This will refresh it if needed
            let token = self.token().await?;

            // Send the request to get user profile
            match request::get!("me").send(token).await {
                Ok(SpotifyResponse { body, .. }) => Ok(serde_json::from_str(&body)?),
                Err(err) => Err(err),
            }
        }
    }

    /// Get the current user's top artists or tracks based on calculated affinity.
    ///
    /// <N> Is the number of items to return per page.
    /// <T> Is the type of items to return. [Artist, Track]
    fn get_user_top_items<T, const N: usize>(
        &self,
        time_range: TimeRange,
    ) -> Result<Paginated<TopItems<T>, Self, N>, Error>
    where
        T: IntoUserTopItemType + Deserialize<'static> + Debug + Clone + PartialEq
    {
        validate_scope(self.scopes(), &[scopes::USER_TOP_READ])?;
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/me/top/{}?time_range={}&limit={}&offset={}",
                API_BASE_URL,
                T::into_top_item_type(),
                time_range,
                N,
                0,
            )),
            None,
            |c: &TopItems<T>| {
                (c.next.clone(), c.previous.clone())
            },
        ))
    }
}
