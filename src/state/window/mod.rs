pub mod library;
pub mod modal;

use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Padding, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
};
use rspotify::model::{CursorBasedPage, Page};

use super::InnerState;

#[derive(Default, Debug, Clone, Copy)]
pub enum Window {
    #[default]
    Library,
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
        }
    }
}

pub trait PageRow {
    type Row;
    type Container: StatefulWidget;

    fn page_row(&self) -> Self::Row;
    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State;
    fn page_widths<'a>(items: impl Iterator<Item = &'a Self>) -> Vec<Constraint>
    where
        Self: 'a;
    fn page_container(items: Vec<Self::Row>, width: Vec<Constraint>) -> Self::Container;
}

pub trait Paginatable {
    type Container: StatefulWidget;
    type Row;

    fn paginated(&self, offset: Option<u32>, index: usize) -> Paginated<Self::Container>;
    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State;
    fn page_rows(&self) -> Vec<Self::Row>;
}

impl<T: PageRow> Paginatable for Page<T> {
    type Container = T::Container;
    type Row = T::Row;

    fn paginated(&self, _offset: Option<u32>, index: usize) -> Paginated<Self::Container> {
        Paginated {
            length: self.items.len(),
            container: T::page_container(self.page_rows(), T::page_widths(self.items.iter())),
            state: Self::page_state(index),
            index,
            total: self.total,
            offset: self.offset,
            page_size: self.limit,
        }
    }

    fn page_rows(&self) -> Vec<Self::Row> {
        self.items.iter().map(|v| v.page_row()).collect()
    }

    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State {
        T::page_state(index)
    }
}

impl<T: PageRow> Paginatable for CursorBasedPage<T> {
    type Container = T::Container;
    type Row = T::Row;

    fn paginated(&self, offset: Option<u32>, index: usize) -> Paginated<Self::Container> {
        Paginated {
            length: self.items.len(),
            container: T::page_container(self.page_rows(), T::page_widths(self.items.iter())),
            state: Self::page_state(index),
            index,
            total: self.total.unwrap_or_default(),
            offset: offset.unwrap_or_default(),
            page_size: self.limit,
        }
    }

    fn page_rows(&self) -> Vec<Self::Row> {
        self.items.iter().map(|v| v.page_row()).collect()
    }

    fn page_state(index: usize) -> <Self::Container as StatefulWidget>::State {
        T::page_state(index)
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

            let page_block = Block::new().padding(Padding::top(1));
            Line::from(vec![
                Span::from(
                    (0..current.saturating_sub(1))
                        .map(|_| '•')
                        .collect::<String>(),
                )
                .dark_gray(),
                Span::from("•").green(),
                Span::from((0..pages - current).map(|_| '•').collect::<String>()).dark_gray(),
            ])
            .centered()
            .render(page_block.inner(vert[1]), buf);

            vert[0]
        } else {
            area
        };

        let block = if scrollable {
            // TODO: Display scrollbar
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(self.length).position(self.index);
            StatefulWidget::render(
                scrollbar,
                layout.inner(Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
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
