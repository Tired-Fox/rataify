mod profile;

use crate::api::users::profile::CurrentUserProfileBuilder;
use crate::auth::OAuth;

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
}