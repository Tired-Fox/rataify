use ratatui::widgets::StatefulWidget;

use crate::state::InnerState;

#[derive(Debug, Clone, Copy)]
pub enum Modal {

}

impl StatefulWidget for Modal {
    type State = InnerState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
                
    }
}
