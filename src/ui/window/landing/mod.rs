use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Layout, Margin, Rect}, symbols::border, text::Span, widgets::{
        block::{Block, Padding, Position, Title}, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget, Wrap 
    }
};
use ratatui_image::Image;

use crate::{
    state::{window::landing::{Cover, Landing}, Loading}, ui::{components::OpenInSpotify, PaginationProgress, COLORS}, Locked, Shared
};

mod artist;
mod playlist;
mod album;
mod show;
mod audiobook;

lazy_static::lazy_static! {
    pub static ref HTML_UNICODE: regex::Regex = regex::Regex::new("&#(?:(?<decimal>[0-9]+)|x(?<hex>[0-9a-fA-F]+));").unwrap();
    pub static ref HTML_TAG: regex::Regex = regex::Regex::new("</?[abis][^>]*>").unwrap();
}

impl Widget for &mut Landing {
fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Landing::None => {},
            Landing::Playlist{ pages, state, playlist, cover, section } => {
                playlist::render(area, buf, playlist, pages, state, section, cover);
            },
            Landing::Album{ pages, state, album, cover, section } => {
                album::render(area, buf, album, pages, state, section, cover);
            },
            Landing::Show{ pages, state, show, cover, section } => {
                show::render(area, buf, show, pages, state, section, cover);
            },
            Landing::Audiobook{ pages, state, audiobook, cover, section } => {
                audiobook::render(area, buf, audiobook, pages, state, section, cover);
            },
            Landing::Artist { top_tracks, state, section, albums, artist, cover, landing_section } => {
                let top_tracks = top_tracks.lock().unwrap();
                let artist = &*artist.lock().unwrap();
                artist::render(area, buf, artist, top_tracks.as_slice(), albums, state, section, landing_section, cover);
            }
        }
    }
}

static INFO_CUTOFF: u16 = 18;
static COMPACT: u16 = 21;

fn render_landing(
    area: Rect,
    buf: &mut Buffer,
    title: String,
    cover: Shared<Locked<Loading<Cover>>>,
) -> (Rect, Rect)
{
    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .padding(Padding::symmetric(1, 1))
        .title(
            Title::from(title)
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        );

    (&block).render(area, buf);
    let inner = block.inner(area);

    let hoz = Layout::horizontal([
        Constraint::Length(30),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(inner);

    // RENDER ARTIST INFORMATION
    let info_area = match cover.lock().unwrap().as_ref() {
        Loading::Some(cover) => {
            let lyt = Layout::vertical([
                Constraint::Length(14),
                if area.height < INFO_CUTOFF {
                    Constraint::Length(0)
                } else {
                    Constraint::Fill(1)
                },
            ])
            .split(hoz[0]);
            Image::new(cover.image.as_ref()).render(lyt[0], buf);
            lyt[1]
        }
        Loading::Loading => Layout::vertical([
            Constraint::Length(14),
            if area.height < INFO_CUTOFF {
                Constraint::Length(0)
            } else {
                Constraint::Fill(1)
            },
        ])
        .split(hoz[0])[1],
        Loading::None => hoz[0],
    };

    (info_area, hoz[2])
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Bold,
    Italic,
    Link(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextPart<'a> {
    Text(Span<'a>),
    Link(OpenInSpotify)
}

pub struct Description<'a> {
    parts: Vec<TextPart<'a>>,
    wrap: Wrap
}

impl<'a> Description<'a> {
    pub fn new(parts: Vec<TextPart<'a>>) -> Self {
        Self {
            parts,
            wrap: Wrap { trim: false }
        }
    }
}

impl<'a> Widget for Description<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
        // Take newlines into consideration. Break to new lines based on this.
        let mut row = area.y;
        let mut col = area.x;
        let wrapping = !self.wrap.trim;

        for part in self.parts {
            match part {
                TextPart::Text(span) =>  {
                    for char in span.content.chars() {
                        if char == '\n' {
                            if !wrapping {
                                return
                            }
                            row += 1;
                            col = 0;
                            continue
                        }

                        buf.get_mut(col, row).set_char(char).set_style(span.style);
                        col += 1;
                        if col >= area.width && wrapping {
                            row += 1;
                            col = 0;
                        }
                    }
                },
                TextPart::Link(link) => {
                    // first and last chars get the style for a hyperlink. Everything else is
                    // printed normally
                    // \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\
                    if link.label.len() >= 2 {
                        buf.get_mut(col, row).set_symbol(format!("\x1B]8;;{}\x07{}", link.url(), link.label.get(0..1).unwrap()).as_str());
                        col += 1;
                        if col >= area.width && wrapping {
                            row += 1;
                            col = 0;
                        }
                        for char in link.label.chars().take(link.label.len()-1).skip(1) {
                            if char == '\n' {
                                if !wrapping {
                                    return
                                }
                                row += 1;
                                col = 0;
                                continue
                            }

                            buf.get_mut(col, row).set_char(char);

                            col += 1;
                            if col >= area.width && wrapping {
                                row += 1;
                                col = 0;
                            }
                        }    
                        buf.get_mut(col, row).set_symbol(format!("{}\x1B]8;;\x07", link.label.get(link.label.len()-1..link.label.len()).unwrap()).as_str());
                    } else if link.label.len() == 1 {
                        buf.get_mut(col, row).set_symbol(format!("\x1B]8;;{}\x07{}\x1B]8;;\x07", link.url(), link.label).as_str());
                    }
                },
            }
        }
    }
}
