//! Port of `corpora/cmudict/cmudict.py::return_dict()`.
//!
//! The Python code consumes a *generated* dictionary built by
//! `prep_corpus_to_dict.py` from the raw `corpora/cmudict/cmudict` file. We
//! reproduce that generation exactly so the resulting map — and crucially its
//! `dict.keys().__str__()` string representation, which `count_syllables` and
//! `markov_haiku` abuse as a substring haystack — is byte-identical.
//!
//! Two reproduced quirks:
//!   * The generator's EOF `readline()` returns `""`, which it processes before
//!     stopping, so the map contains a trailing empty-string key: `'': ['']`.
//!   * `str(dict_keys)` renders each key with Python's `repr()`, so a key
//!     containing an apostrophe (and no double quote) is rendered with double
//!     quotes, e.g. `"ZYUGANOV'S"`.

use crate::text_util::read_text;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub struct CmuDict {
    /// word -> list of pronunciation strings (matches `return_dict()`).
    pub map: HashMap<String, Vec<String>>,
    /// keys in first-seen (insertion) order, mirroring Python dict ordering.
    pub order: Vec<String>,
    /// `dict.keys().__str__()` — e.g. `dict_keys(['A', 'A.', ..., '']) `.
    pub keys_repr: String,
    /// `dict.keys().__str__().lower()`.
    pub keys_repr_lower: String,
}

/// Reproduces Python's `repr()` of a `str` for the limited charset that appears
/// in cmudict keys (ASCII letters/digits/punctuation, including apostrophes and
/// double quotes).
fn py_repr(s: &str) -> String {
    let has_single = s.contains('\'');
    let has_double = s.contains('"');
    // Python prefers single quotes; switches to double only if the string has a
    // single quote and no double quote.
    let quote = if has_single && !has_double { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if is_py_printable(c) => out.push(c),
            c if (c as u32) < 0x100 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if (c as u32) < 0x10000 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push_str(&format!("\\U{:08x}", c as u32)),
        }
    }
    out.push(quote);
    out
}

fn is_py_printable(c: char) -> bool {
    // Sufficient for the ASCII keys present; printable ASCII is 0x20..=0x7e.
    // Non-ASCII (none occur here) is conservatively treated as printable.
    if c.is_ascii() {
        (' '..='~').contains(&c)
    } else {
        !c.is_control()
    }
}

static CMUDICT: Lazy<CmuDict> = Lazy::new(|| {
    let text = read_text("corpora/cmudict/cmudict");
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    // Reproduce prep_corpus_to_dict.py: readline() yields each line plus a final
    // "" at EOF; splitting on '\n' (text ends in a newline) gives exactly that
    // trailing empty element. Process each element; stop after an empty line
    // (matches `if line == "" or line == "\n": more = False`, which breaks AFTER
    // processing that line).
    for line in text.split('\n') {
        // `line.replace('\n','').split(" ")` — line already has no '\n'.
        let parts: Vec<&str> = line.split(' ').collect();
        let word = parts[0].to_string();
        // `' '.join(this_line[2:])`
        let pron = if parts.len() > 2 {
            parts[2..].join(" ")
        } else {
            String::new()
        };
        match map.get_mut(&word) {
            Some(v) => v.push(pron),
            None => {
                order.push(word.clone());
                map.insert(word, vec![pron]);
            }
        }
        if line.is_empty() {
            break;
        }
    }

    let keys_repr = {
        let mut s = String::from("dict_keys([");
        for (i, k) in order.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&py_repr(k));
        }
        s.push_str("])");
        s
    };
    let keys_repr_lower = keys_repr.to_lowercase();

    CmuDict {
        map,
        order,
        keys_repr,
        keys_repr_lower,
    }
});

pub fn get() -> &'static CmuDict {
    &CMUDICT
}

/// `cmudict.return_dict()` — the word -> pronunciations map.
pub fn dict() -> &'static HashMap<String, Vec<String>> {
    &get().map
}
