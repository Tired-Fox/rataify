pub mod device;
pub mod actions;
pub mod goto;

use actions::Actions;
use device::Device;
use goto::GoTo;
use ratatui::{layout::{Constraint, Layout}, widgets::StatefulWidget};

use crate::state::InnerState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modal {
    Devices,
    Actions,
    GoTo,
}

impl StatefulWidget for Modal {
    type State = InnerState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        match self {
            Self::Devices => {
                Device.render(area, buf, state)
            },
            Self::Actions => {
                Actions.render(area, buf, state)
            }
            Self::GoTo => {
                GoTo.render(area, buf, state)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center
}

pub fn clamp(length: usize, max: u16, percent: f32) -> u16 {
    (length as u16).min((max as f32 * percent) as u16)
}

pub fn modal_layout(area: ratatui::prelude::Rect, width: Constraint, height: Constraint, position: ModalPosition) -> ratatui::prelude::Rect {
    match position {
        ModalPosition::TopLeft => {
            let vert = Layout::vertical([height, Constraint::Fill(1)]).split(area);
            let horz = Layout::horizontal([width, Constraint::Fill(1)]).split(vert[0]);
            horz[0]
        },
        ModalPosition::TopRight => {
            let vert = Layout::vertical([Constraint::Fill(1), height]).split(area);
            let horz = Layout::horizontal([width, Constraint::Fill(1)]).split(vert[1]);
            horz[0]
        },
        ModalPosition::BottomLeft => {
            let vert = Layout::vertical([height, Constraint::Fill(1)]).split(area);
            let horz = Layout::horizontal([Constraint::Fill(1), width]).split(vert[0]);
            horz[1]
        },
        ModalPosition::BottomRight => {
            let vert = Layout::vertical([Constraint::Fill(1), height]).split(area);
            let horz = Layout::horizontal([Constraint::Fill(1), width]).split(vert[1]);
            horz[1]
        },
        ModalPosition::Center => {
            let vert = Layout::vertical([Constraint::Fill(1), height, Constraint::Fill(1)]).split(area);
            let horz = Layout::horizontal([Constraint::Fill(1), width, Constraint::Fill(1)]).split(vert[1]);
            horz[1]
        },
    }
}
