use crate::api::users::profile::{CurrentUserProfileBuilder, GetUserProfileBuilder};
use crate::auth::OAuth;

mod profile;
mod follow;

pub struct UsersBuilder<'a> {
    oauth: &'a mut OAuth,
}

impl<'a> UsersBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self { oauth }
    }

    /// Get current user profile
    ///
    /// # Scope
    /// Optional: user-read-private, user-read-email
    pub fn current_user_profile(self) -> CurrentUserProfileBuilder<'a> {
        CurrentUserProfileBuilder::new(self.oauth)
    }


    /// Get a user's public profile
    pub fn get_users_profile<S: Into<String>>(self, user_id: S) -> GetUserProfileBuilder<'a> {
        GetUserProfileBuilder::new(self.oauth, user_id)
    }

    /// Get a user's top items of either `tracks` or `artists`
    ///
    /// # Scope
    /// user-top-read
    /// TODO: Check to see if this is a valid endpoint
    // #[cfg(feature = "user-top-read")]
    // pub fn get_users_top_items<I: AsTopItem>(self) -> UsersTopItemsBuilder<'a, I> {
    //     UsersTopItemsBuilder::<I>::new(self.oauth, I::top_item())
    // }

    /// Get followed artists
    ///
    /// # Scope
    /// user-follow-read
    #[cfg(feature = "user-follow-read")]
    pub fn get_followed_artists(self) -> follow::FollowedArtistsBuilder<'a> {
        follow::FollowedArtistsBuilder::new(self.oauth)
    }

    /// Follow a playlist
    ///
    /// # Scope
    /// playlist-modify-public, playlist-modify-private
    #[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
    pub fn follow_playlist<S: Into<String>>(self, playlist_id: S) -> follow::FollowPlaylistBuilder<'a> {
        follow::FollowPlaylistBuilder::new(self.oauth, playlist_id.into())
    }

    /// Unfollow a playlist
    ///
    /// # Scope
    /// playlist-modify-public, playlist-modify-private
    #[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
    pub fn unfollow_playlist<S: Into<String>>(self, playlist_id: S) -> follow::UnfollowPlaylistBuilder<'a> {
        follow::UnfollowPlaylistBuilder::new(self.oauth, playlist_id.into())
    }
}