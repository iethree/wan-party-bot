//! Port of `markov_haiku_discord.py` (the parts reachable from `gen_haiku`).
//! Builds first/second-order Markov maps over a syllable-aware corpus and emits a
//! 5/7/5 haiku. Only ever reached through the never-synced `/haiku` command.

use crate::cmudict;
use crate::count_syllables::count_syllables;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

/// `prep_training`: normalize, lowercase, split, and keep only words that appear
/// as a substring of `cmudict.keys().__str__().lower()`.
fn prep_training(raw_haiku: &str) -> Vec<String> {
    let cmu = cmudict::get();
    raw_haiku
        .replace('\n', " ")
        .replace(['.', ',', '/'], "")
        .to_lowercase()
        .split_whitespace()
        .filter(|w| cmu.keys_repr_lower.contains(*w))
        .map(|w| w.to_string())
        .collect()
}

fn map_word_to_word(corpus: &[String]) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    if corpus.is_empty() {
        return m;
    }
    let limit = corpus.len() - 1;
    for (index, word) in corpus.iter().enumerate() {
        if index < limit {
            m.entry(word.clone()).or_default().push(corpus[index + 1].clone());
        }
    }
    m
}

fn map_2_words_to_word(corpus: &[String]) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    if corpus.len() < 2 {
        return m;
    }
    let limit = corpus.len() - 2;
    for (index, word) in corpus.iter().enumerate() {
        if index < limit {
            let key = format!("{} {}", word, corpus[index + 1]);
            m.entry(key).or_default().push(corpus[index + 2].clone());
        }
    }
    m
}

/// `random.choice([])` raises `IndexError('Cannot choose from an empty sequence')`;
/// reproduced as this error string so the `/haiku` fallback message matches.
const EMPTY_CHOICE: &str = "IndexError('Cannot choose from an empty sequence')";

fn random_word(corpus: &[String]) -> Result<(String, i64), String> {
    let mut rng = rand::thread_rng();
    // Python recurses on every >4-syllable pick; an all->4-syllable corpus exhausts
    // the recursion limit (~1000) and raises RecursionError, which the /haiku caller
    // turns into its fallback. Cap attempts so we surface that instead of hanging.
    for _ in 0..1000 {
        let word = corpus
            .choose(&mut rng)
            .ok_or_else(|| EMPTY_CHOICE.to_string())?
            .clone();
        let num_syls = count_syllables(&word);
        if num_syls > 4 {
            continue;
        }
        return Ok((word, num_syls));
    }
    Err("RecursionError('maximum recursion depth exceeded')".to_string())
}

fn word_after(
    prefix: &str,
    suffix_map: &HashMap<String, Vec<String>>,
    current_syls: i64,
    target_syls: i64,
) -> Vec<String> {
    let mut accepted = Vec::new();
    if let Some(suffixes) = suffix_map.get(prefix) {
        for candidate in suffixes {
            let num_syls = count_syllables(candidate);
            if current_syls + num_syls <= target_syls {
                accepted.push(candidate.clone());
            }
        }
    }
    accepted
}

/// Returns the last `n` elements (clamped), mirroring Python's `list[-2:]`.
fn last_n(v: &[String], n: usize) -> Vec<String> {
    let start = v.len().saturating_sub(n);
    v[start..].to_vec()
}

fn haiku_line(
    suffix_map_1: &HashMap<String, Vec<String>>,
    suffix_map_2: &HashMap<String, Vec<String>>,
    corpus: &[String],
    mut end_prev_line: Vec<String>,
    target_syls: i64,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut rng = rand::thread_rng();
    let mut line = "2/3";
    let mut line_syls: i64 = 0;
    let mut current_line: Vec<String> = Vec::new();

    if end_prev_line.is_empty() {
        // build first line
        line = "1";
        let (word, num_syls) = random_word(corpus)?;
        current_line.push(word.clone());
        line_syls += num_syls;
        let mut word_choices = word_after(&word, suffix_map_1, line_syls, target_syls);
        while word_choices.is_empty() {
            let prefix = corpus
                .choose(&mut rng)
                .ok_or_else(|| EMPTY_CHOICE.to_string())?
                .clone();
            word_choices = word_after(&prefix, suffix_map_1, line_syls, target_syls);
        }
        let word = word_choices.choose(&mut rng).unwrap().clone();
        let num_syls = count_syllables(&word);
        line_syls += num_syls;
        current_line.push(word);
        if line_syls == target_syls {
            end_prev_line.extend(last_n(&current_line, 2));
            return Ok((current_line, end_prev_line));
        }
    } else {
        // build lines 2 and 3
        current_line.extend(end_prev_line.iter().cloned());
    }

    loop {
        let n = current_line.len();
        let prefix = format!("{} {}", current_line[n - 2], current_line[n - 1]);
        let mut word_choices = word_after(&prefix, suffix_map_2, line_syls, target_syls);
        while word_choices.is_empty() {
            // Python `random.randint(0, len(corpus) - 2)` raises ValueError on a
            // <2-word corpus; signal it rather than underflowing usize.
            if corpus.len() < 2 {
                return Err("ValueError('empty range for randrange()')".to_string());
            }
            let index = rng.gen_range(0..=corpus.len() - 2);
            let prefix = format!("{} {}", corpus[index], corpus[index + 1]);
            word_choices = word_after(&prefix, suffix_map_2, line_syls, target_syls);
        }
        let word = word_choices.choose(&mut rng).unwrap().clone();
        let num_syls = count_syllables(&word);
        if line_syls + num_syls > target_syls {
            continue;
        } else if line_syls + num_syls < target_syls {
            current_line.push(word);
            line_syls += num_syls;
        } else {
            current_line.push(word);
            break;
        }
    }

    let new_end_prev = last_n(&current_line, 2);
    let final_line = if line == "1" {
        current_line.clone()
    } else {
        // current_line[2:]
        if current_line.len() > 2 {
            current_line[2..].to_vec()
        } else {
            Vec::new()
        }
    };
    Ok((final_line, new_end_prev))
}

/// `gen_haiku(training_file)` -> three lines (each a list of words).
///
/// In the original this is unguarded: an empty/degenerate corpus makes
/// `random.choice` / `random.randint` raise, and the `/haiku` command's try/except
/// turns that into the "Inspiration eludes me..." fallback. Here the failure is
/// surfaced as `Err(repr)` carrying Python's `repr(e)` so the caller can reproduce
/// that message verbatim.
pub fn gen_haiku(training_file: &str) -> Result<Vec<Vec<String>>, String> {
    let corpus = prep_training(training_file);
    let suffix_map_1 = map_word_to_word(&corpus);
    let suffix_map_2 = map_2_words_to_word(&corpus);

    let mut final_lines: Vec<Vec<String>> = Vec::new();
    let end_prev_line: Vec<String> = Vec::new();
    let (first_line, end_prev_line1) =
        haiku_line(&suffix_map_1, &suffix_map_2, &corpus, end_prev_line, 5)?;
    final_lines.push(first_line);
    let (line2, end_prev_line2) =
        haiku_line(&suffix_map_1, &suffix_map_2, &corpus, end_prev_line1, 7)?;
    final_lines.push(line2);
    let (line3, _end_prev_line3) =
        haiku_line(&suffix_map_1, &suffix_map_2, &corpus, end_prev_line2, 5)?;
    final_lines.push(line3);

    Ok(final_lines)
}
