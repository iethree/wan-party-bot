//! Port of `sing.py`. Rotates through the lines of `data/rick.txt`, one per call,
//! wrapping around. The Python module keeps a global `saved_index`; we guard the
//! same state behind a mutex so concurrent serenity tasks behave like the
//! single-threaded Python event loop.

use crate::text_util::read_text;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static RICKS: Lazy<Vec<String>> = Lazy::new(|| {
    // `open('./data/rick.txt').read().split('\n')` — a trailing newline therefore
    // yields a final empty element, faithfully reproduced here.
    read_text("./data/rick.txt")
        .split('\n')
        .map(|s| s.to_string())
        .collect()
});

static SAVED_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

pub fn sing_to_me() -> String {
    let mut idx = SAVED_INDEX.lock().unwrap();
    if *idx == RICKS.len() {
        *idx = 0;
    }
    let line = format!("🎶  *{}*  🎶", RICKS[*idx]);
    *idx += 1;
    line
}
