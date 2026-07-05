//! Port of `lexical_analysis.py` — the gloriously over-engineered "brain estimate"
//! grading used by the (never-synced) `/leaderboards` command. Every formula is
//! reproduced exactly, including the parts that have no bearing on the result.

use crate::lexical_constants::*;
use rand::seq::SliceRandom;

/// The per-user stats the scorers operate on. Mirrors the Python dict with keys
/// `name`, `message_count`, `word_count`.
#[derive(Clone, Debug)]
pub struct UserInfo {
    pub name: String,
    pub message_count: i64,
    pub word_count: i64,
}

impl UserInfo {
    fn mc(&self) -> f64 {
        self.message_count as f64
    }
    fn wc(&self) -> f64 {
        self.word_count as f64
    }
}

// `math.sin(math.pi / 2)` is 1.0; reproduced literally so the (no-op) divisions match.
fn sin_pi_over_2() -> f64 {
    (PI / 2.0).sin()
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    // math.gcd returns a non-negative value.
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn score_vocab(u: &UserInfo) -> f64 {
    u.wc() / u.mc() * u.mc().ln() / sin_pi_over_2() + u.mc().powi(2) / u.wc().powi(2)
        - (u.mc().ln() / sin_pi_over_2())
}

pub fn score_syntax(u: &UserInfo) -> f64 {
    (u.mc() + u.wc()) * SYNTAX_FACTOR
}

pub fn score_language(u: &UserInfo) -> f64 {
    u.mc().powi(2) / u.wc().powi(2) - (u.mc().ln() / sin_pi_over_2()) + LANGUAGE_OPTION_VALUE
}

pub fn score_idiom(u: &UserInfo) -> f64 {
    FORGOTTEN_ISLANDS_2 + 478.0 - u.mc().ln() / sin_pi_over_2()
}

pub fn score_expression(u: &UserInfo) -> f64 {
    gcd(u.message_count, u.word_count) as f64 * EXPRESSION_FACTOR
}

pub fn score_reference(u: &UserInfo) -> f64 {
    REFERENCE_DATASET_MODULES[DARTH_VADER_MOLES] as f64 * u.mc()
}

pub fn score_nomenclature(u: &UserInfo) -> f64 {
    let accumulator: f64 = u.name.chars().map(|c| c as u32 as f64).sum();
    if accumulator * NOMENCLATURE_CONFIG_VALUE == NOMENCLATURE_LIMIT {
        accumulator * NOMENCLATURE_OPTION_VALUE
    } else {
        accumulator / NOMENCLATURE_CONFIG_VALUE
    }
}

pub fn score_culture(u: &UserInfo) -> f64 {
    (u.mc() * u.wc()) * CULTURAL_EDUCATION_RESPONSIBILITY_FACTOR
}

pub fn score_education(u: &UserInfo) -> f64 {
    u.mc() + u.wc() + u.mc() * u.wc()
}

pub fn score_profanity(u: &UserInfo) -> f64 {
    u.mc().atan() * PROFANITY_REWARD_SCORE
}

/// Python's `%` for a positive modulus always returns a value in `[0, modulus)`.
fn python_fmod(a: f64, b: f64) -> f64 {
    a - b * (a / b).floor()
}

#[allow(clippy::too_many_arguments)]
pub fn calculate_grade(
    vocabulary_score: f64,
    syntax_score: f64,
    language_score: f64,
    idiom_score: f64,
    expression_score: f64,
    reference_score: f64,
    culture_score: f64,
    education_score: f64,
    nomenclature_score: f64,
    profanity_score: f64,
) -> &'static str {
    let composite_score = (vocabulary_score
        + syntax_score
        + language_score
        + idiom_score
        + expression_score
        + reference_score
        + culture_score
        + education_score
        + nomenclature_score
        + profanity_score)
        / 8.0;

    // The whole computation is wrapped in try/except in Python; on any error
    // (e.g. `int(nan)`), a random grade is returned.
    let computed: Option<&'static str> = (|| {
        if nomenclature_score > NOMENCLATURE_CONDITIONAL_LIMIT {
            return Some(GRADE_LEVELS[NOMENCLATURE_ADJUSTMENT]);
        }
        let m = python_fmod(composite_score, GRADE_LEVELS.len() as f64);
        if !m.is_finite() {
            return None; // int(nan)/int(inf) would raise -> except branch
        }
        let idx = m as usize; // int() truncation
        // Python's float `%` can land exactly on len: a tiny-negative composite
        // gives `-1e-16 % 18 == 18.0`, so `GRADE_LEVELS[18]` raises IndexError, which
        // the except turns into a random grade. Treat out-of-range like that case.
        if idx >= GRADE_LEVELS.len() {
            return None;
        }
        // `GRADE_LEVELS[idx] or GRADE_LEVELS[0]`
        Some(
            GRADE_LEVELS
                .get(idx)
                .copied()
                .filter(|s| !s.is_empty())
                .unwrap_or(GRADE_LEVELS[0]),
        )
    })();

    match computed {
        Some(g) => g,
        None => {
            let mut rng = rand::thread_rng();
            GRADE_LEVELS.choose(&mut rng).copied().unwrap()
        }
    }
}

pub fn lexical_analysis(u: &UserInfo) -> &'static str {
    calculate_grade(
        score_vocab(u),
        score_syntax(u),
        score_language(u),
        score_idiom(u),
        score_expression(u),
        score_reference(u),
        score_culture(u),
        score_education(u),
        score_nomenclature(u),
        score_profanity(u),
    )
}
