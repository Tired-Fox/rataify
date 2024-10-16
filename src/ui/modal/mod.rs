use crossterm::event::{KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Layout, Rect}, style::{Color, Style}, symbols::border, widgets::{block::Title, Block, Cell, Clear, Padding, Row, StatefulWidget, Table, TableState, Widget}};

use super::COLORS;

pub mod devices;
pub mod actions;
pub mod goto;
pub mod add_to_playlist;
pub mod artists;

trait KeyToString {
    fn key_to_string(&self) -> String;
}

impl KeyToString for KeyEvent {
    fn key_to_string(&self) -> String {
        let mut buff = String::new();
        if self.modifiers & KeyModifiers::CONTROL != KeyModifiers::NONE {
            buff.push_str("ctrl+");
        }
        if self.modifiers & KeyModifiers::ALT != KeyModifiers::NONE {
            buff.push_str("alt+");
        }

        if self.modifiers & KeyModifiers::SHIFT != KeyModifiers::NONE {
            match self.code {
                KeyCode::Char(_) => {},
                _ => buff.push_str("shift+"),
            }
        }

        buff.push_str(&match self.code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::F(f) => format!("F{}", f),
            KeyCode::Media(media) => format!("{:?}", media),
            KeyCode::Modifier(_) => String::new(),
            other => format!("{:?}", other),
        });

        buff
    }
}

pub fn render_modal<const N: usize, I: IntoIterator<Item=[String; N]>>(area: Rect, buf: &mut Buffer, title: &str, rows: I) {
    let mut longest_parts: [usize; N] = [0; N];

    // Rernder in bottom right corner
    // [{key}] {title}
    let mut count = 0;
    let list = rows.into_iter().map(|parts| {
        count += 1;
        let cells = parts.into_iter().enumerate().map(|(i, part)| {
            if part.len() > longest_parts[i] {
                longest_parts[i] = part.len();
            }

            Cell::from(part)
        });

        Row::new(cells)
    })
        .collect::<Table>()
        .block(Block::bordered()
            //.borders(Borders::TOP | Borders::LEFT)
            .border_set(border::ROUNDED)
            .padding(Padding::symmetric(1, 1))
            .title(Title::from(title).alignment(Alignment::Center))
        )
        .widths(longest_parts.iter().map(|l| Constraint::Length(*l as u16)))
        .column_spacing(2);

    let hoz = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length((longest_parts.iter().sum::<usize>() as u16 + 6).max(title.len() as u16 + 2)),
        Constraint::Length(1),
    ])
        .split(area);

    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(((count + 4) as u16).min(area.height - 4)),
        Constraint::Length(1),
    ])
        .split(hoz[1]);

    Clear.render(vert[1], buf);
    Widget::render(list, vert[1], buf);
}

pub fn render_modal_with_state<const N: usize, I: IntoIterator<Item=[String; N]>>(area: Rect, buf: &mut Buffer, title: &str, rows: I, state: &mut TableState) {
    let mut longest_parts: [usize; N] = [0; N];

    // Rernder in bottom right corner
    // [{key}] {title}
    let mut count = 0;
    let list = rows.into_iter().map(|parts| {
        count += 1;
        let cells = parts.into_iter().enumerate().map(|(i, part)| {
            if part.len() > longest_parts[i] {
                longest_parts[i] = part.len();
            }

            Cell::from(part)
        });

        Row::new(cells)
    })
        .collect::<Table>()
        .block(Block::bordered()
            //.borders(Borders::TOP | Borders::LEFT)
            .border_set(border::ROUNDED)
            .padding(Padding::symmetric(1, 1))
            .title(Title::from(title).alignment(Alignment::Center))
        )
        .highlight_style(COLORS.highlight)
        .widths(longest_parts.iter().map(|l| Constraint::Length(*l as u16)))
        .column_spacing(2);

    let hoz = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length((longest_parts.iter().sum::<usize>() as u16 + 6).max(title.len() as u16 + 2)),
        Constraint::Length(1),
    ])
        .split(area);

    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(((count + 4) as u16).min(area.height - 4)),
        Constraint::Length(1),
    ])
        .split(hoz[1]);

    Clear.render(vert[1], buf);
    StatefulWidget::render(list, vert[1], buf, state);
}
