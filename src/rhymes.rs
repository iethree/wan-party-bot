//! Port of `rhymes.py`. Reachable only through the never-synced `/rhyme` command.
//!
//! Faithfully reproduces: the misnamed `vowel_location` (actually consonant
//! indices), the sequential try-append of `results`, Python negative indexing,
//! the `rhyming_type` variable mutating across pronunciations, the KeyError (for
//! an unknown word) propagating out while an IndexError returns an empty list.

use crate::cmudict;

/// What `rhyme()` returns: either the single-entry dict `{WORD: [matches]}`, or
/// the empty list `[]` produced by the internal `except IndexError`.
pub enum RhymeResult {
    Dict { word: String, matches: Vec<String> },
    Empty,
}

/// Python negative-capable index. Returns None for an out-of-range access
/// (Python would raise IndexError).
fn py_index(v: &[i64], i: i64) -> Option<i64> {
    let len = v.len() as i64;
    let idx = if i < 0 { len + i } else { i };
    if idx < 0 || idx >= len {
        None
    } else {
        Some(v[idx as usize])
    }
}

/// Python `v[start:]` with a possibly-negative start.
fn py_slice_from<'a>(v: &'a [&'a str], start: i64) -> &'a [&'a str] {
    let len = v.len() as i64;
    let eff = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    };
    &v[eff as usize..]
}

fn strip_digits(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_digit()).collect()
}

/// `get_phoneme_count_to_rhyme`. Only the location bookkeeping affects the result.
fn get_phoneme_count_to_rhyme(phoneme_input: &[&str]) -> Vec<i64> {
    // indices of phonemes that contain a stress digit (vowels)
    let mut syllable_location: Vec<usize> = Vec::new();
    // indices of phonemes that contain NO digit (the misnamed "vowel_location")
    let mut vowel_location: Vec<usize> = Vec::new();

    for (i, phoneme) in phoneme_input.iter().enumerate() {
        for d in '0'..='9' {
            if phoneme.contains(d) {
                syllable_location.push(i);
                break;
            }
            if d == '9' {
                // reached only when no digit 0-9 was present
                vowel_location.push(i);
            }
        }
    }

    let len = phoneme_input.len() as i64;
    let mut results: Vec<i64> = Vec::new();
    // Sequential appends emulating Python's `try: ... except IndexError: pass`: the
    // first unavailable index stops the rest.
    'append: {
        let Some(&x) = vowel_location.last() else {
            break 'append;
        };
        results.push(len - x as i64);
        let s = syllable_location.len();
        if s < 1 {
            break 'append;
        }
        results.push(len - syllable_location[s - 1] as i64);
        if s < 2 {
            break 'append;
        }
        results.push(len - syllable_location[s - 2] as i64);
        if s < 3 {
            break 'append;
        }
        results.push(len - syllable_location[s - 3] as i64);
    }
    results
}

/// `rhyme(word, rhyming_type=0)`.
///
/// Returns `Err(repr)` for the KeyError case (word not in the dictionary), which in
/// Python propagates out of `rhyme()` to the caller's try/except as
/// `KeyError('WORD')`. The carried string reproduces Python's `repr(e)` so the
/// `/rhyme` error message matches verbatim.
pub fn rhyme(word: &str, rhyming_type: i64) -> Result<RhymeResult, String> {
    let cmu = cmudict::get();
    let upper = word.to_uppercase();
    let upper_e = format!("{upper}E");

    // `DICTIONARY[word.upper()]` — KeyError propagates (not caught by except IndexError).
    let pronunciation = match cmu.map.get(&upper) {
        Some(p) => p,
        None => return Err(format!("KeyError('{upper}')")),
    };

    let mut matches: Vec<String> = Vec::new();
    let mut rhyming_type = rhyming_type;

    // The try block; an internal IndexError -> return RhymeResult::Empty.
    let ok = (|| -> Option<()> {
        for a_pronunciation in pronunciation {
            let word_phonemes: Vec<&str> = a_pronunciation.split(' ').collect();
            let phoneme_target = get_phoneme_count_to_rhyme(&word_phonemes);
            let word_phonemes_count = word_phonemes.len() as i64;

            loop {
                let mut matches_count = 0i64;
                let rhyming_target: i64 = if rhyming_type == 0 {
                    py_index(&phoneme_target, -1)? // phoneme_target[-1]
                } else if (phoneme_target.len() as i64) >= rhyming_type - 1 {
                    py_index(&phoneme_target, rhyming_type - 1)?
                } else {
                    py_index(&phoneme_target, -1)?
                };

                let slice = py_slice_from(&word_phonemes, word_phonemes_count - rhyming_target);
                let target_sounds = strip_digits(&slice.join(" "));

                for potential_match in &cmu.order {
                    if let Some(variations) = cmu.map.get(potential_match) {
                        for match_variation in variations {
                            if strip_digits(match_variation).ends_with(&target_sounds) {
                                if *potential_match == upper || *potential_match == upper_e {
                                    continue;
                                }
                                matches.push(potential_match.clone());
                                matches_count += 1;
                            }
                        }
                    }
                }

                if rhyming_type == 1 {
                    break;
                } else if matches_count < 1 {
                    rhyming_type -= 1;
                } else {
                    break;
                }
            }
        }
        Some(())
    })();

    match ok {
        Some(()) => Ok(RhymeResult::Dict {
            word: upper,
            matches,
        }),
        None => Ok(RhymeResult::Empty), // except IndexError: return []
    }
}
