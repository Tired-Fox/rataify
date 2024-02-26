use std::fmt::{Debug, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::{ImageSource, StatefulProtocol};
use ratatui_image::{Resize, StatefulImage};

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

    static ref PATTERNS: [Vec<char>; 3] = [
        vec!['-', '•'],
        vec!['▘', '▝', '▖', '▗'],
        vec!['◢', '◣', '◤', '◥'],
    ];
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

    pub fn change_image<P>(&mut self, path: P)
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
        f.render_stateful_widget(image, area, &mut protocol);
    }
}

#[derive(Debug, Default)]
pub struct NowPlayingState {
    pub cover: Vec<String>,
    pub name: String,
    pub artist: String,
    pub album: String,
}

impl NowPlayingState {
    pub fn now_playing<P, S1, S2, S3>(&mut self, path: P, name: S1, artist: S2, album: S3)
    where
        P: AsRef<Path>,
        S1: Display,
        S2: Display,
        S3: Display,
    {
        self.name = name.to_string();
        self.album = album.to_string();
        self.artist = artist.to_string();
        self.cover = self.generate_cover();
    }

    pub fn cover(&self, height: usize) -> String {
        let height = height - 2;
        let width: usize = (height as f32 * 2.5) as usize;
        let mut output = format!("┌{}┐\n", "─".repeat(width));
        output.push_str(
            format!(
                "{}",
                self.cover
                    .iter()
                    .skip((245 - height) / 2)
                    .take(height)
                    .map(|r| format!(
                        "│{}│",
                        r.chars()
                            .skip((245 - width) / 2)
                            .take(width)
                            .collect::<String>()
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
            .as_str(),
        );
        output.push_str(format!("\n└{}┘", "─".repeat(width)).as_str());
        output
    }

    fn generate_cover(&self) -> Vec<String> {
        // TODO: Better algorithms based on pseudo random seeds from title, artist, and album
        let mut hasher = DefaultHasher::default();
        self.artist.hash(&mut hasher);

        let first = hasher.finish();
        let mut rng_artist = StdRng::seed_from_u64(first);
        let pattern: usize = rng_artist.gen_range(0..PATTERNS.len());
        let mut pattern = PATTERNS[pattern].clone();
        if rng_artist.gen() {
            pattern.push(' ')
        }

        let scale = rng_artist.gen_range(0..pattern.len() * 12);
        // Pick random characters from pattern
        let picks = rng_artist.gen_range(0..(pattern.len() * scale));

        self.name.hash(&mut hasher);
        let mut rng_name = StdRng::seed_from_u64(hasher.finish());
        let pattern: Vec<char> = (0..picks)
            .map(|_| pattern[rng_name.gen_range(0..pattern.len())])
            .collect();

        let step = rng_name.gen_range(1..(PATTERNS.len() / 2).max(2));

        // Infinite wrapping pattern
        let size = pattern.len();
        let mut pattern = pattern.iter().cycle().step_by(step);
        self.album.hash(&mut hasher);
        let mut rng_album = StdRng::seed_from_u64(hasher.finish());

        // 245x245 random char sample
        (0..245)
            .map(|_| {
                (0..245)
                    .map(|_| pattern.nth(rng_album.gen_range(0..size)).unwrap())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
    }
}

pub struct State {
    pub counter: i64,
    pub now_playing: Option<NowPlayingState>,
}

impl Default for State {
    fn default() -> Self {
        let path = PathBuf::from("Bling-Bang-Bang-Born.jpg");

        // FIXME: Temp default song information
        let mut now_playing = NowPlayingState::default();
        now_playing.now_playing(
            path,
            "Bling-Bang-Bang-Born",
            "Creepy Nuts",
            "Bling-Bang-Bang-Born",
        );

        Self {
            counter: 0,
            now_playing: Some(now_playing),
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
