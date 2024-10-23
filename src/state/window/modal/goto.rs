use std::str::FromStr;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Constraint, widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, StatefulWidget, Table, TableState, Widget}};
use rspotify::{clients::BaseClient, model::{AlbumId, ArtistId, PlaylistId, ShowId, Type}, AuthCodePkceSpotify};

use crate::{action::{Action, Open}, app::ContextSender, state::InnerState, uri::Uri, Error};

use super::{modal_layout, ModalPosition};

pub struct GoTo;
impl GoTo {
    pub async fn handle_action(action: Action, spotify: AuthCodePkceSpotify, state: InnerState, context: ContextSender) -> Result<(), Error> {
        // TODO: if landing and landing matches currently playing context skip adding `Open Context`

        #[allow(clippy::single_match)]
        match action {
            Action::Key(key) => match key {
                KeyEvent { code: KeyCode::Char('L'), .. } => {
                    context.send_action(Action::Open(Open::Library))?;
                    context.send_action(Action::Close)?;
                }
                KeyEvent { code: KeyCode::Char('C'), .. } => {
                    let ctx = state.playback.lock().unwrap().as_ref().and_then(|p| p.context.as_ref().map(|c| c.uri.clone()));
                    if let Some(ctx) = ctx {
                        match Uri::from_str(ctx.as_str())?.ty {
                            Type::Album => if let Ok(album) = spotify.album(AlbumId::from_uri(ctx.as_str())?, None).await {
                                context.send_action(Action::Open(Open::album(&album.into())))?;
                            }
                            Type::Artist => if let Ok(artist) = spotify.artist(ArtistId::from_uri(ctx.as_str())?).await {
                                context.send_action(Action::Open(Open::artist(&artist.into())))?;
                            }
                            Type::Show => if let Ok(show) = spotify.get_a_show(ShowId::from_uri(ctx.as_str())?, None).await {
                                context.send_action(Action::Open(Open::show(&show.into())))?;
                            }
                            Type::Playlist => if let Ok(playlist) = spotify.playlist(PlaylistId::from_uri(ctx.as_str())?, None, None).await {
                                context.send_action(Action::Open(Open::playlist(&playlist.into())))?;
                            }
                            other => return Err(Error::custom(format!("unknown context: {other:?}")))
                        }
                        context.send_action(Action::Close)?;
                    }
                },
                _ => {}
            }
            _ => {}
        }
        Ok(())
    }
}

impl StatefulWidget for GoTo {
    type State = InnerState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let mut lines = 3;
        let mut pos = TableState::default();

        let mut rows = vec![
            Row::new(vec![
                Cell::from("L"),
                Cell::from("Library"),
            ])
        ];

        let widths = vec![
            Constraint::Length(1),
            Constraint::Fill(7),
        ];

        if state.playback.lock().unwrap().as_ref().map(|p| p.context.is_some()).unwrap_or_default() {
            rows.push(Row::new(vec![
                Cell::new("C"),
                Cell::new("Context"),
            ]));
            lines += 1;
        }

        let layout = modal_layout(area, Constraint::Length(15), Constraint::Length(lines), ModalPosition::BottomRight);

        Clear.render(layout, buf);

        let block = Block::new()
            .title("Goto")
            .padding(Padding::horizontal(2))
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        Widget::render(&block, layout, buf);

        let table = Table::new(rows, widths);
        StatefulWidget::render(table, block.inner(layout), buf, &mut pos);
    }
}
