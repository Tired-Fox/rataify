use std::fmt::{Debug, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
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
        vec!['…', '.'],
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

    corners: Flag, // 1 = tl + br, 2 = tr + bl
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
struct Flag(u8);
impl BitAnd for Flag {
    type Output = Flag;
    fn bitand(self, rhs: Self) -> Self::Output {
        Flag(self.0 & rhs.0)
    }
}
impl BitOr for Flag {
    type Output = Flag;
    fn bitor(self, rhs: Self) -> Self::Output {
        Flag(self.0 | rhs.0)
    }
}
impl BitOrAssign for Flag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}
impl BitAndAssign for Flag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}
const TLBR: Flag = Flag(1);
const TRBL: Flag = Flag(2);

impl NowPlayingState {
    pub fn now_playing<S1, S2, S3>(&mut self, name: S1, artist: S2, album: S3)
    where
        S1: Display,
        S2: Display,
        S3: Display,
    {
        self.corners = Flag::default();

        self.name = name.to_string();
        self.album = album.to_string();
        self.artist = artist.to_string();
        self.generate_cover();
    }

    // TODO: Randomize border based on title and artist.
    //  Corners based on artist, sides based on title
    pub fn cover(&self, height: usize) -> String {
        let height = height - 2;
        let width: usize = (height as f32 * 2.5) as usize;
        let mut output = format!(
            "{}{}{}\n",
            if self.corners & TLBR == TLBR { "┌─" } else { "  " },
            " ".repeat(width-2),
            if self.corners & TRBL == TRBL { "─┐" } else { "  " },
        );
        output.push_str(
            format!(
                "{}",
                self.cover
                    .iter()
                    .skip((50 - height) / 2)
                    .take(height)
                    .map(|r| format!(
                        " {} ",
                        r.chars()
                            .skip((50 - width) / 2)
                            .take(width)
                            .collect::<String>(),
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
            .as_str(),
        );
        output.push_str(format!(
            "\n{}{}{}",
            //'┌', '┐', '└', '┘'
            if self.corners & TRBL == TRBL { "└─" } else { "  " },
            " ".repeat(width-2),
            if self.corners & TLBR == TLBR { "─┘" } else { "  " },
        ).as_str());
        output
    }

    fn generate_cover(&mut self) {
        let mut hasher = DefaultHasher::default();
        self.album.hash(&mut hasher);

        let mut rng = StdRng::seed_from_u64(hasher.finish());
        let pattern: usize = rng.gen_range(0..PATTERNS.len());
        let mut pattern = PATTERNS[pattern].clone();
        if rng.gen() {
            pattern.push(' ')
        }

        let scale = rng.gen_range(pattern.len()..pattern.len() * 12);
        // Pick random characters from pattern
        let picks = rng.gen_range(pattern.len()..pattern.len()+(pattern.len() * scale));

        let pattern: Vec<char> = (0..picks)
            .map(|_| pattern[rng.gen_range(0..pattern.len())])
            .collect();

        let step = rng.gen_range(1..(PATTERNS.len() / 2).max(2));

        // Infinite wrapping pattern
        let size = pattern.len();
        let mut pattern = pattern.iter().cycle().step_by(step);

        // 50x50 random char sample
        self.cover = (0..50)
            .map(|_| {
                (0..50)
                    .map(|_| pattern.nth(rng.gen_range(0..size)).unwrap())
                    .collect::<String>()
            })
            .collect::<Vec<String>>();

        if rng.gen() {
            self.corners |= TLBR;
        }
        if rng.gen() {
            self.corners |= TRBL
        }
    }
}

pub struct State {
    pub counter: u8,
    pub now_playing: Option<NowPlayingState>,
}

impl Default for State {
    fn default() -> Self {
        let path = PathBuf::from("Bling-Bang-Bang-Born.jpg");

        // FIXME: Temp default song information
        let mut now_playing = NowPlayingState::default();
        let song = SONGS[0];
        now_playing.now_playing(song.0, song.1, song.2);

        Self {
            counter: 0,
            now_playing: Some(now_playing),
        }
    }
}

static SONGS: [(&str, &str, &str);3] = [
    ("Bling-Bang-Bang-Born", "Creepy Nuts", "Bling-Bang-Bang-Born"),
    ("Time for two", "RADWIMPS", "Susume (Motion Picture Soundtrack)"),
    ("Tamaki", "RADWIMPS, Toaka", "Susume (Motion Picture Soundtrack)"),
];

impl State {
    fn update_song(&mut self) {
        if let Some(now_playing) = &mut self.now_playing {
            let song = SONGS[self.counter as usize];
            now_playing.now_playing(song.0, song.1, song.2);
        }
    }

    pub(crate) fn next(&mut self) {
        self.counter = (self.counter + 1) % 3;
        self.update_song();
    }

    pub(crate) fn previous(&mut self) {
        let counter = self.counter as i8 - 1;
        if counter < 0 {
            self.counter = 3_i8.saturating_add(counter) as u8;
        } else {
            self.counter = counter as u8;
        }

        self.update_song();
    }
}
