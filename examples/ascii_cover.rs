use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crossterm::terminal;
use rand::{rngs::StdRng, Rng, SeedableRng};

lazy_static::lazy_static! {
    static ref PATTERNS: [Vec<char>; 3] = [
        vec!['-', '•'],
        vec!['▘', '▝', '▖', '▗'],
        vec!['◢', '◣', '◤', '◥'],
    ];
}

fn main() {
    let artist = "Creepy Nuts";
    let name = "Bling-Bang-Bang-Born";
    let album = "Bling-Bang-Bang-Born";

    // let artist = "Brennan Story";
    // let name = "Twin flame";
    // let album = "Twin flame";

    let mut hasher = DefaultHasher::default();
    artist.hash(&mut hasher);

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

    name.hash(&mut hasher);
    let mut rng_name = StdRng::seed_from_u64(hasher.finish());
    let pattern: Vec<char> = (0..picks)
        .map(|_| pattern[rng_name.gen_range(0..pattern.len())])
        .collect();
    println!("{:?}", pattern);

    let step = rng_name.gen_range(1..(PATTERNS.len() / 2).max(2));

    // Infinite wrapping pattern
    let size = pattern.len();
    let mut pattern = pattern
        .iter()
        .cycle()
        .skip(rng_name.gen_range(0..245))
        .step_by(step);
    album.hash(&mut hasher);
    let mut rng_album = StdRng::seed_from_u64(hasher.finish());

    // 100 x 100 = 1000 chars
    // 10 rows
    let sample = (0..245)
        .map(|_| {
            (0..245)
                .map(|_| pattern.nth(rng_album.gen_range(0..size)).unwrap())
                .collect::<String>()
        })
        .collect::<Vec<String>>();

    // TODO: Centered in pattern
    //
    // 30 x 30
    // 5x5
    // 3x3

    // println!("{}", sample.join("\n"));
    println!("{:?}", terminal::size());

    let print_pattern = |height: usize| {
        let height = height - 2;
        let width: usize = (height as f32 * 2.5) as usize;
        println!("{width}");
        println!("┌{}┐", "─".repeat(width));
        println!(
            "{}",
            sample
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
        );
        println!("└{}┘", "─".repeat(width));
    };

    print_pattern(20);
    print_pattern(8);
    print_pattern(5);
}
