//! File-reading helpers that reproduce Python text-mode semantics.
//!
//! Python's `open(path, encoding="utf8").read()` performs *universal newline*
//! translation: `\r\n` and lone `\r` both become `\n`. Rust's
//! `std::fs::read_to_string` does NOT. Several data files (notably
//! `data/dick.txt`) use CRLF terminators, and the corpora are split on `"\n\n"`,
//! so without translation the splits would differ. These helpers normalize
//! newlines the same way Python does.

use std::fs;

/// Equivalent of `open(path, encoding="utf8").read()` in Python text mode.
///
/// Reads the file, decodes as UTF-8 (lossily, matching Python's tolerance for
/// the well-formed data files here), and applies universal-newline translation.
///
/// Panics if the file cannot be read, mirroring Python raising at module import
/// time when a required data file is missing.
pub fn read_text(path: &str) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("could not read {path}: {e}"));
    let s = String::from_utf8_lossy(&bytes);
    normalize_newlines(&s)
}

/// Universal-newline translation: `\r\n` -> `\n`, then lone `\r` -> `\n`.
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Equivalent of `open(path).readlines()` on universal-newline text: splits while
/// KEEPING the trailing `\n` on every line except possibly the last (matching
/// Python `readlines`). Returns an empty vec for an empty file.
pub fn read_lines_keepends(path: &str) -> Vec<String> {
    let text = read_text(path);
    if text.is_empty() {
        return Vec::new();
    }
    // `split_inclusive('\n')` keeps the terminator, exactly like readlines().
    text.split_inclusive('\n').map(|s| s.to_string()).collect()
}
