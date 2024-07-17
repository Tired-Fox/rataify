use std::fmt::Display;
use tupy::api::{Uri, Resource, UserResource};

use ratatui::{widgets::Widget, style::Style};

static BASE_URL: &str = "https://open.spotify.com";

pub struct OpenInSpotify {
    pub uri: Uri,
    pub label: String
}
impl OpenInSpotify {
    pub fn playlist<S: Display, L: Display>(id: S, label: L) -> Self {
        Self {
            uri: Uri::playlist(id),
            label: label.to_string(),
        }
    }
    pub fn album<S: Display, L: Display>(id: S, label: L) -> Self {
        Self {
            uri: Uri::album(id),
            label: label.to_string(),
        }
    }
    pub fn artist<S: Display, L: Display>(id: S, label: L) -> Self {
        Self {
            uri: Uri::artist(id),
            label: label.to_string(),
        }
    }
    pub fn show<S: Display, L: Display>(id: S, label: L) -> Self {
        Self {
            uri: Uri::show(id),
            label: label.to_string(),
        }
    }
    pub fn liked_tracks<S: Display, L: Display>(id: S, label: L) -> Self {
        Self {
            uri: Uri::collection(id),
            label: label.to_string(),
        }
    }
    pub fn saved_episodes<S: Display, L: Display>(id: S, label: L) -> Self {
        Self {
            uri: Uri::collection_your_episodes(id),
            label: label.to_string(),
        }
    }

    pub fn url(&self) -> String {
        match self.uri.resource() {
            Resource::Artist => format!("{BASE_URL}/artist/{}", self.uri.id()),
            Resource::Album => format!("{BASE_URL}/album/{}", self.uri.id()),
            Resource::Playlist => format!("{BASE_URL}/playlist/{}", self.uri.id()),
            Resource::User(user_resource) => match user_resource {
                UserResource::None => format!("{BASE_URL}/user/{}", self.uri.id()),
                UserResource::Collection => format!("{BASE_URL}/collection/tracks"),
                UserResource::CollectionYourEpisodes => format!("{BASE_URL}/collection/your-episodes"),
            },
            Resource::Show => format!("{BASE_URL}/show/{}", self.uri.id()),
            _ => panic!("Unsupported uri for open in spotify")
        }
    }

    pub fn ansi(&self) -> String {
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", self.url(), self.label)
    }
}

impl Widget for OpenInSpotify {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        buf.set_string(area.x, area.y, self.ansi(), Style::default())
    }
}
