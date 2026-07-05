//! Port of `count_syllables_discord.py`.
//!
//! Reproduces two quirks faithfully:
//!   * Membership is tested with `word.upper() not in cmudict.keys().__str__()` —
//!     a *substring* test against the giant repr string, NOT a real key lookup.
//!   * The syllable count comes from iterating the characters of the first
//!     pronunciation string and counting digits (`phoneme[-1].isdigit()` over
//!     single characters), which equals the number of ARPAbet stress markers.

use crate::cmudict;

/// Python's `string.punctuation`.
const PUNCT: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

pub fn count_syllables(words: &str) -> i64 {
    let replaced = words.replace('-', " ");
    let lowered = replaced.to_lowercase();
    let cmu = cmudict::get();
    let mut num_sylls = 0i64;

    // Python's `str.split()` whitespace set includes the C0 information separators
    // U+001C..=U+001F, which `char::is_whitespace` (Unicode White_Space) omits.
    let is_py_space = |c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c);
    for token in lowered.split(is_py_space).filter(|t| !t.is_empty()) {
        let mut word: String = token
            .trim_matches(|c: char| PUNCT.contains(c))
            .to_string();
        if word.ends_with("'s") || word.ends_with("’s") {
            let n = word.chars().count();
            word = word.chars().take(n.saturating_sub(2)).collect();
        }
        let upper = word.to_uppercase();
        // Substring test against `dict.keys().__str__()`.
        if !cmu.keys_repr.contains(&upper) {
            continue;
        }
        // `cmudict[word.upper()][0]` then count digit characters. A missing key
        // raises KeyError in Python, caught and skipped (num unchanged).
        if let Some(prons) = cmu.map.get(&upper) {
            if let Some(first) = prons.first() {
                num_sylls += first.chars().filter(|c| c.is_ascii_digit()).count() as i64;
            }
        }
    }
    num_sylls
}
