pub mod library;
pub mod modal;

pub mod landing;

use landing::Landing;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget
    },
};
use rspotify::model::{CursorBasedPage, Page};

use super::InnerState;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Window {
    #[default]
    Library,
    Landing,
}

impl StatefulWidget for Window {
    type State = InnerState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().dark_gray());

        Widget::render(&block, area, buf);

        let win = *state.window.lock().unwrap();
        match win {
            Self::Library => library::Library.render(block.inner(area), buf, state),
            Self::Landing => Landing.render(block.inner(area), buf, state),
        }
    }
}

pub type CellBuilder = Box<dyn Fn(String) -> Cell<'static>>;
pub trait PageRow {
    fn page_row(&self) -> Vec<(String, Option<CellBuilder>)>;
    fn page_widths(widths: Vec<usize>) -> Vec<Constraint>;
}

pub trait Paginatable {
    type Container: StatefulWidget;
    type Row;

    fn paginated(&self, offset: Option<u32>, index: usize) -> Paginated<Self::Container>;
    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State;
}

impl<T: PageRow> Paginatable for Page<T> {
    type Container = Table<'static>;
    type Row = Row<'static>;

    fn paginated(&self, _offset: Option<u32>, index: usize) -> Paginated<Self::Container> {
        let mut widths: Vec<usize> = Vec::new();
        let mut rows = Vec::new();

        for item in self.items.iter() {
            rows.push(Row::new(
                item.page_row()
                    .into_iter()
                    .enumerate()
                    .map(|(i, (col, style))| {
                        match widths.get(i).copied() {
                            Some(len) => if col.len() > len {
                                if let Some(len) = widths.get_mut(i) {
                                    *len = col.len();
                                }
                            } 
                            None => widths.push(i)
                        }

                        match style {
                            Some(s) => s(col),
                            None => Cell::from(col)
                        }
                    })
            ))
        }

        Paginated {
            length: self.items.len(),
            state: Self::page_state(index),
            container: Table::new(rows, T::page_widths(widths)).highlight_style(Style::default().yellow()),
            index,
            total: self.total,
            offset: self.offset,
            page_size: self.limit,
        }
    }

    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State {
        TableState::default().with_selected(Some(index))
    }
}

impl<T: PageRow> Paginatable for CursorBasedPage<T> {
    type Container = Table<'static>;
    type Row = Row<'static>;

    fn paginated(&self, offset: Option<u32>, index: usize) -> Paginated<Self::Container> {
        let mut widths: Vec<usize> = Vec::new();
        let mut rows = Vec::new();

        for item in self.items.iter() {
            rows.push(Row::new(
                item.page_row()
                    .into_iter()
                    .enumerate()
                    .map(|(i, (col, style))| {
                        match widths.get(i).copied() {
                            Some(len) => if col.len() > len {
                                if let Some(len) = widths.get_mut(i) {
                                    *len = col.len();
                                }
                            } 
                            None => widths.push(i)
                        }

                        match style {
                            Some(s) => s(col),
                            None => Cell::from(col)
                        }
                    })
            ))
        }

        Paginated {
            length: self.items.len(),
            container: Table::new(rows, T::page_widths(widths)).highlight_style(Style::default().yellow()),
            state: Self::page_state(index),
            index,
            total: self.total.unwrap_or_default(),
            offset: offset.unwrap_or_default(),
            page_size: self.limit,
        }
    }

    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State {
        TableState::default().with_selected(Some(index))
    }
}

pub struct Paginated<T: StatefulWidget> {
    /// Current page of items
    pub container: T,
    pub state: T::State,
    /// Current number of items being displayed
    pub length: usize,
    /// Currently selected item
    pub index: usize,
    /// Total number of items
    pub total: u32,
    /// Current item offset
    pub offset: u32,
    /// Items per page
    pub page_size: u32,
}

impl<T: StatefulWidget> StatefulWidget for Paginated<T> {
    type State = T::State;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        // If multiple pages display last two lines and pagination progress
        // If more items than can fit last column is scroll bar
        let current = (self.offset as f32 / self.page_size as f32).ceil() as u32 + 1;
        let pages = (self.total as f32 / self.page_size as f32).ceil() as u32;

        let pagination = pages > 1;

        let max_height = if pagination {
            area.height.saturating_sub(2)
        } else {
            area.height
        };

        let scrollable = self.length > max_height as usize;

        let layout = if pagination {
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).split(area);

            let min = current.saturating_sub(5).max(1);
            let max = current.saturating_add(5).min(pages);

            let page_block = Block::new().padding(Padding::top(1));
            Line::from(vec![
                Span::from(if current > 6 { "… " } else { "" }).dark_gray(),
                Span::from(
                    (min..current)
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
                .dark_gray(),
                Span::from(format!(" {current} ")).green(),
                Span::from((current+1..=max).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")).dark_gray(),
                Span::from(if (pages-current) > 5 { " …" } else { "" }).dark_gray(),
            ])
            .centered()
            .render(page_block.inner(vert[1]), buf);

            vert[0]
        } else {
            area
        };

        let block = if scrollable {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(self.length).position(self.index);
            StatefulWidget::render(
                scrollbar,
                layout,
                buf,
                &mut scrollbar_state,
            );

            Block::default().padding(Padding::right(1))
        } else {
            Block::default()
        };

        self.container.render(block.inner(layout), buf, state);
    }
}
