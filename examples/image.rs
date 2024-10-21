use std::io::stdout;

use crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};
use rataify::Error;
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Layout}, Terminal};
use ratatui_image::{picker::Picker, Resize, StatefulImage};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    let mut picker = Picker::new((7, 16));

    // Load an image with the image crate.
    let dyn_img = image::ImageReader::open("./assets/Wistoria-Wand-and-Sword.jpg")?.decode().unwrap();

    // Create the Protocol which will be used by the widget.
    let mut image = picker.new_resize_protocol(dyn_img);

    loop {
        terminal.draw(|f| {
            let img = StatefulImage::new(None).resize(Resize::Fit(None));

            let hoz = Layout::horizontal([Constraint::Fill(1), Constraint::Length(50), Constraint::Fill(1)]).split(f.area());
            let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(25), Constraint::Fill(1)]).split(hoz[1]);

            f.render_stateful_widget(
                img,
                vert[1],
                &mut image,
            );
        })?;
    }
}
