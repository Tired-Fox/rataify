use ratatui::{layout::Constraint, style::{Style, Stylize}, widgets::{Block, BorderType, Borders, Clear, List, ListState, Padding, StatefulWidget, Widget}};
use rspotify::model::Device as SpotifyDevice;

use crate::state::InnerState;

use super::{clamp, modal_layout, ModalPosition};

#[derive(Default, Debug, Clone)]
pub struct DeviceState {
    pub devices: Vec<SpotifyDevice>,
    pub play: Option<bool>,
    pub index: usize,
}

impl DeviceState {
    pub fn next(&mut self) {
        if self.index < self.devices.len() - 1 {
            self.index += 1;
        }
    }

    pub fn prev(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    pub fn select(&self) -> Option<SpotifyDevice> {
        self.devices.get(self.index).cloned()
    }

    pub fn reset(&mut self, devices: Vec<SpotifyDevice>, play: Option<bool>) {
        self.play = play;
        self.index = 0;
        self.devices = devices;
    }
}

pub struct Device;
impl StatefulWidget for Device {
    type State = InnerState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let mut longest = 0;
        let (devices, index) = {
            let dstate = state.devices.lock().unwrap();
            let devices = dstate.devices.iter().map(|d| {
                if d.name.len() > longest {
                    longest = d.name.len();
                }
                d.name.clone()
            }).collect::<Vec<_>>();
            (devices, dstate.index)
        };

        let width = clamp((longest + 4).max(11), area.width, 0.6);
        let height = clamp(devices.len() + 2, area.height, 0.6);

        let layout = modal_layout(area, Constraint::Length(width), Constraint::Length(height), ModalPosition::Center);

        Clear.render(layout, buf);

        let block = Block::new()
            .title("Devices")
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        (&block).render(layout, buf);

        let mut pos = ListState::default().with_selected(Some(index));
        let list = List::new(devices)
            .highlight_style(Style::default().yellow());
        StatefulWidget::render(list, block.inner(layout), buf, &mut pos);
    }
}
