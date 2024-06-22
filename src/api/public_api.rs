use crate::Error;

use super::{
    flow::AuthFlow,
    response::{NewReleases, Paginated},
    DefaultResponse, API_BASE_URL,
};

pub trait PublicApi: AuthFlow {
    /// Get a list of new album releases featured in Spotify
    /// (shown, for example, on a Spotify player’s “Browse” tab).
    fn get_new_releases<const N: usize>(
        &self,
    ) -> Result<Paginated<NewReleases, Self, N>, Error> {
        Ok(Paginated::new(
            self.clone(),
            Some(format!(
                "{}/browse/new-releases?limit={}&offset={}",
                API_BASE_URL,
                N,
                0,
            )),
            None,
            |c: &NewReleases| {
                (c.albums.next.clone(), c.albums.previous.clone())
            },
        ))
    }
}
