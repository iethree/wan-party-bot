//! Port of `dick.py` — the literary/quote corpora.
//!
//! Each corpus file is read (with universal-newline translation) and split on
//! blank lines (`"\n\n"`), except the yoda and dwarf corpora which split on
//! single newlines.

use crate::text_util::read_text;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;

fn split_paragraphs(path: &str) -> Vec<String> {
    read_text(path).split("\n\n").map(|s| s.to_string()).collect()
}

fn split_lines(path: &str) -> Vec<String> {
    read_text(path).split('\n').map(|s| s.to_string()).collect()
}

static DICKS: Lazy<Vec<String>> = Lazy::new(|| split_paragraphs("./data/dick.txt"));
static DICKENSES: Lazy<Vec<String>> = Lazy::new(|| split_paragraphs("./data/bleak-house.txt"));
static WILLIES: Lazy<Vec<String>> = Lazy::new(|| split_paragraphs("./data/willy.txt"));
static SUMMAS: Lazy<Vec<String>> = Lazy::new(|| split_paragraphs("./data/summa.txt"));
static JANES: Lazy<Vec<String>> = Lazy::new(|| split_paragraphs("./data/jane.txt"));
static YODA: Lazy<Vec<String>> = Lazy::new(|| split_lines("./data/clone-wars-quotes.txt"));
static DWARFS: Lazy<Vec<String>> = Lazy::new(|| split_lines("./data/dwarfs.txt"));

fn pool_for(name: &str) -> &'static Vec<String> {
    // Mirrors the `dudes` dict. A missing key would KeyError in Python; here it
    // panics for the same fail-fast behavior (callers only ever pass known names).
    match name {
        "dick" => &DICKS,
        "dickens" => &DICKENSES,
        "willy" => &WILLIES,
        "thomas" => &SUMMAS,
        "jane" => &JANES,
        "yoda" => &YODA,
        "dwarf" => &DWARFS,
        other => panic!("unknown quote corpus: {other}"),
    }
}

pub fn get_yoda_quote() -> String {
    let mut rng = rand::thread_rng();
    format!("> {}", YODA.choose(&mut rng).unwrap())
}

pub fn get_dwarf_quote() -> String {
    let mut rng = rand::thread_rng();
    format!("> {}", DWARFS.choose(&mut rng).unwrap())
}

/// `get_random_quote(name)` — keep choosing until the quote's *character* length
/// is in `[100, 800]`. (Python `len()` counts code points, so we use `chars()`.)
pub fn get_random_quote(name: &str) -> String {
    let pool = pool_for(name);
    let mut rng = rand::thread_rng();
    loop {
        let choice = pool.choose(&mut rng).unwrap();
        let len = choice.chars().count();
        if (100..=800).contains(&len) {
            return choice.clone();
        }
    }
}
