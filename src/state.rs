use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::{Resize, StatefulImage};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::{ImageSource, StatefulProtocol};

lazy_static::lazy_static! {
    pub static ref PICKER: Mutex<Picker> = {
        #[cfg(windows)]
        return Mutex::new(Picker::new((8, 16)));

        #[cfg(not(windows))]
        return Mutex::new({
            let mut picker = Picker::from_termios().unwrap();
            picker.guess_protocol().unwrap();
            picker
        });
    };
}

pub struct ImageState {
    pub image_source: ImageSource,
    pub image_protocol: Box<dyn StatefulProtocol>,
}

impl Debug for ImageState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImageState()")
    }
}

impl ImageState {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let picker = &mut PICKER.lock().unwrap();
        let mut dyn_img = image::io::Reader::open(path).unwrap().decode().unwrap();
        dyn_img = dyn_img.resize(250, 250, image::imageops::FilterType::Gaussian);

        Self {
            image_source: ImageSource::new(dyn_img.clone(), picker.font_size),
            image_protocol: picker.new_resize_protocol(dyn_img.clone()),
        }
    }

    pub fn change_image<P>(&mut self, path: P )
    where
        P: AsRef<Path>,
    {
        let picker = &mut PICKER.lock().unwrap();
        let mut dyn_img = image::io::Reader::open(path).unwrap().decode().unwrap();
        dyn_img = dyn_img.resize(250, 250, image::imageops::FilterType::Gaussian);
        self.image_source = ImageSource::new(dyn_img.clone(), picker.font_size);
        self.image_protocol = picker.new_resize_protocol(dyn_img.clone());
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let image = StatefulImage::new(None).resize(Resize::Fit);
        let mut protocol = self.image_protocol.clone();
        f.render_stateful_widget(
            image,
            area,
            &mut protocol,
        );
    }
}

#[derive(Debug, Default)]
pub struct NowPlayingState {
    pub cover: Option<ImageState>,
    pub name: String,
}

impl NowPlayingState {
    pub fn now_playing<P>(&mut self, path: P, name: String)
    where
        P: AsRef<Path>,
    {
        match self.cover {
            Some(ref mut cover) => {
                cover.change_image(path);
            }
            None => {
                self.cover = Some(ImageState::new(path));
            }
        }
        self.name = name;
    }
}

pub struct State {
    pub counter: i64,
    pub now_playing: Option<NowPlayingState>,
}

impl Default for State {
    fn default() -> Self {
        let path = PathBuf::from("Bling-Bang-Bang-Born.jpg");
        Self {
            counter: 0,
            now_playing: Some(NowPlayingState {
                cover: Some(ImageState::new(path.as_path())),
                name: String::from("Bling-Bang-Bang-Born"),
            })
        }
    }
}

impl State {
    pub(crate) fn increment(&mut self) {
        if let Some(counter) = self.counter.checked_add(1) {
            self.counter = counter;
        }
    }

    pub(crate) fn decrement(&mut self) {
        if let Some(counter) = self.counter.checked_sub(1) {
            self.counter = counter;
        }
    }
}
