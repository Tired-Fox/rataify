use std::collections::HashMap;
use cfg_if::cfg_if;

use serde_json::json;

use crate::{Error, SpotifyRequest, SpotifyResponse};
use crate::auth::OAuth;
use crate::model::follow::FollowedArtists;
use crate::model::user::{UserProfile, UserPublicProfile};
use crate::model::Wrapped;

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
    pub fn get_current_user_profile(self) -> GetCurrentUserProfileBuilder<'a> {
        GetCurrentUserProfileBuilder::new(self.oauth)
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

    /// Follow a playlist
    ///
    /// # Scope
    /// playlist-modify-public, playlist-modify-private
    #[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
    pub fn follow_playlist<S: Into<String>>(self, playlist_id: S) -> FollowPlaylistBuilder<'a> {
        FollowPlaylistBuilder::new(self.oauth, playlist_id.into())
    }

    /// Unfollow a playlist
    ///
    /// # Scope
    /// playlist-modify-public, playlist-modify-private
    #[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
    pub fn unfollow_playlist<S: Into<String>>(self, playlist_id: S) -> UnfollowPlaylistBuilder<'a> {
        UnfollowPlaylistBuilder::new(self.oauth, playlist_id.into())
    }

    /// Get followed artists
    ///
    /// # Scope
    /// user-follow-read
    #[cfg(feature = "user-follow-read")]
    pub fn get_followed_artists(self) -> GetFollowedArtistsBuilder<'a> {
        GetFollowedArtistsBuilder::new(self.oauth)
    }

    /// Follow artists
    ///
    /// # Scope
    /// user-follow-modify
    #[cfg(feature = "user-follow-modify")]
    pub fn follow_artists<S: IntoIterator<Item=T>, T: Into<String>>(self, ids: S) -> FollowArtistOrUserBuilder<'a> {
        FollowArtistOrUserBuilder::new(self.oauth, FollowVariant::Artist, ids.into_iter().map(|v| v.into()).collect())
    }

    /// Follow users
    ///
    /// # Scope
    /// user-follow-modify
    #[cfg(feature = "user-follow-modify")]
    pub fn follow_users<S: IntoIterator<Item=T>, T: Into<String>>(self, ids: S) -> FollowArtistOrUserBuilder<'a> {
        FollowArtistOrUserBuilder::new(self.oauth, FollowVariant::User, ids.into_iter().map(|v| v.into()).collect())
    }

    /// Unfollow artists
    ///
    /// # Scope
    /// user-follow-modify
    #[cfg(feature = "user-follow-modify")]
    pub fn unfollow_artists<S: IntoIterator<Item=T>, T: Into<String>>(self, ids: S) -> UnfollowArtistOrUserBuilder<'a> {
        UnfollowArtistOrUserBuilder::new(self.oauth, FollowVariant::Artist, ids.into_iter().collect())
    }

    /// Unfollow users
    ///
    /// # Scope
    /// user-follow-modify
    #[cfg(feature = "user-follow-modify")]
    pub fn unfollow_users<S: IntoIterator<Item=T>, T: Into<String>>(self, ids: S) -> UnfollowArtistOrUserBuilder<'a> {
        UnfollowArtistOrUserBuilder::new(self.oauth, FollowVariant::User, ids.into_iter().map(|v| v.into()).collect())
    }

    /// Check if following artists
    ///
    /// # Scope
    /// user-follow-read
    #[cfg(feature = "user-follow-read")]
    pub fn check_follows_artists<S: IntoIterator<Item=T>, T: Into<String>>(self, ids: S) -> CheckFollowsArtistOrUserBuilder<'a> {
        CheckFollowsArtistOrUserBuilder::new(self.oauth, FollowVariant::Artist, ids.into_iter().collect())
    }

    /// Check if following users
    ///
    /// # Scope
    /// user-follow-read
    #[cfg(feature = "user-follow-read")]
    pub fn check_follow_users<S: IntoIterator<Item=T>, T: Into<String>>(self, ids: S) -> CheckFollowsArtistOrUserBuilder<'a> {
        CheckFollowsArtistOrUserBuilder::new(self.oauth, FollowVariant::User, ids.into_iter().map(|v| v.into()).collect())
    }

    /// Check if users follow a playlist
    pub fn check_users_follow_playlist<S, T, U>(self, playlist: S, users: T) -> CheckUsersFollowPlaylistBuilder<'a>
        where
            T: IntoIterator<Item=U>,
            U: Into<String>,
            S: Into<String>,
    {
        CheckUsersFollowPlaylistBuilder::new(self.oauth, playlist.into(), users.into_iter().map(|v| v.into()).collect())
    }
}

pub struct GetUserProfileBuilder<'a> {
    oauth: &'a mut OAuth,
    user_id: String,
}

impl<'a> GetUserProfileBuilder<'a> {
    pub fn new<S: Into<String>>(oauth: &'a mut OAuth, user_id: S) -> Self {
        Self { oauth, user_id: user_id.into() }
    }
}

impl<'a> SpotifyRequest for GetUserProfileBuilder<'a> {
    type Response = UserPublicProfile;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/users/{}", self.user_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetCurrentUserProfileBuilder<'a> {
    oauth: &'a mut OAuth,
}

impl<'a> GetCurrentUserProfileBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self { oauth }
    }
}

impl<'a> SpotifyRequest for GetCurrentUserProfileBuilder<'a> {
    type Response = UserProfile;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct GetFollowedArtistsBuilder<'a> {
    oauth: &'a mut OAuth,
    after: Option<String>,
    limit: Option<usize>,
}

impl<'a> GetFollowedArtistsBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth) -> Self {
        Self {
            oauth,
            after: None,
            limit: None,
        }
    }

    pub fn after(mut self, after: String) -> Self {
        self.after = Some(after);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl<'a> SpotifyRequest for GetFollowedArtistsBuilder<'a> {
    type Response = FollowedArtists;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let mut query = HashMap::from([("type", "artist".to_string())]);
        if let Some(after) = self.after {
            query.insert("after", after);
        }

        if let Some(limit) = self.limit {
            query.insert("limit", limit.to_string());
        }

        let result: Result<Wrapped<FollowedArtists>, Error> = reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/following")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .query(&query)
            .send()
            .to_spotify_response()
            .await;

        result.map(|v| v.unwrap())
    }
}

cfg_if! {
    if #[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))] {

    }
}
#[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
pub struct FollowPlaylistBuilder<'a> {
    oauth: &'a mut OAuth,
    playlist_id: String,
    public: bool,
}

#[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
impl<'a> FollowPlaylistBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, playlist_id: String) -> Self {
        Self {
            oauth,
            playlist_id,
            public: true,
        }
    }

    pub fn public(mut self, public: bool) -> Self {
        self.public = public;
        self
    }
}

#[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
impl<'a> SpotifyRequest for FollowPlaylistBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let body = serde_json::to_string(&json!({
            "public": self.public
        }))?;

        reqwest::Client::new()
            .put(format!("https://api.spotify.com/v1/playlists/{}/followers", self.playlist_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

#[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
pub struct UnfollowPlaylistBuilder<'a> {
    oauth: &'a mut OAuth,
    playlist_id: String,
}

#[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
impl<'a> UnfollowPlaylistBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, playlist_id: String) -> Self {
        Self {
            oauth,
            playlist_id,
        }
    }
}

#[cfg(all(feature = "playlist-modify-public", feature = "playlist-modify-private"))]
impl<'a> SpotifyRequest for UnfollowPlaylistBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .delete(format!("https://api.spotify.com/v1/playlists/{}/followers", self.playlist_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .send()
            .to_spotify_response()
            .await
    }
}

pub enum FollowVariant {
    Artist,
    User,
}

pub struct FollowArtistOrUserBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
    _type: FollowVariant,
}

impl<'a> FollowArtistOrUserBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, _type: FollowVariant, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
            _type,
        }
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.ids.push(id.into());
        self
    }

    pub fn ids<S: IntoIterator<Item=String>>(mut self, ids: S) -> Self {
        self.ids = ids.into_iter().collect();
        self
    }
}

impl<'a> SpotifyRequest for FollowArtistOrUserBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let body = serde_json::to_string(&json!({
            "ids": self.ids
        }))?;

        reqwest::Client::new()
            .put("https://api.spotify.com/v1/me/following")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .query(&[("type", match self._type {
                FollowVariant::Artist => "artist",
                FollowVariant::User => "user",
            })])
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct UnfollowArtistOrUserBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
    _type: FollowVariant,
}

impl<'a> UnfollowArtistOrUserBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, _type: FollowVariant, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
            _type,
        }
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.ids.push(id.into());
        self
    }

    pub fn ids<S: IntoIterator<Item=String>>(mut self, ids: S) -> Self {
        self.ids = ids.into_iter().collect();
        self
    }
}

impl<'a> SpotifyRequest for UnfollowArtistOrUserBuilder<'a> {
    type Response = ();

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        let body = serde_json::to_string(&json!({
            "ids": self.ids
        }))?;

        reqwest::Client::new()
            .delete("https://api.spotify.com/v1/me/following")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .header("Content-Type", "application/json")
            .header("Content-Length", body.len())
            .query(&[("type", match self._type {
                FollowVariant::Artist => "artist",
                FollowVariant::User => "user",
            })])
            .body(body)
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct CheckFollowsArtistOrUserBuilder<'a> {
    oauth: &'a mut OAuth,
    ids: Vec<String>,
    _type: FollowVariant,
}

impl<'a> CheckFollowsArtistOrUserBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, _type: FollowVariant, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
            _type,
        }
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.ids.push(id.into());
        self
    }

    pub fn ids<S: IntoIterator<Item=String>>(mut self, ids: S) -> Self {
        self.ids = ids.into_iter().collect();
        self
    }
}

impl<'a> SpotifyRequest for CheckFollowsArtistOrUserBuilder<'a> {
    type Response = Vec<bool>;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get("https://api.spotify.com/v1/me/following/contains")
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .query(&[
                (
                    "type",
                    match self._type {
                        FollowVariant::Artist => "artist",
                        FollowVariant::User => "user",
                    }.to_string()
                ),
                ("ids", self.ids.join(","))
            ])
            .send()
            .to_spotify_response()
            .await
    }
}

pub struct CheckUsersFollowPlaylistBuilder<'a> {
    oauth: &'a mut OAuth,
    playlist_id: String,
    ids: Vec<String>,
}

impl<'a> CheckUsersFollowPlaylistBuilder<'a> {
    pub fn new(oauth: &'a mut OAuth, playlist_id: String, ids: Vec<String>) -> Self {
        Self {
            oauth,
            ids,
            playlist_id,
        }
    }
}

impl<'a> SpotifyRequest for CheckUsersFollowPlaylistBuilder<'a> {
    type Response = Vec<bool>;

    async fn send(self) -> Result<Self::Response, Error> {
        self.oauth.update().await?;

        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/playlists/{}/followers/contains", self.playlist_id))
            .header("Authorization", self.oauth.token().unwrap().to_header())
            .query(&[("ids", self.ids.join(","))])
            .send()
            .to_spotify_response()
            .await
    }
}
