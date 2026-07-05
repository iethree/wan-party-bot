//! Port of `lexical_constants.py`. The original imports `FORBIDDEN` from
//! `http.client` and then immediately shadows it; both values are dead, but kept
//! here for completeness. Only a handful of these constants are actually read by
//! `lexical_analysis.py`; all are reproduced regardless.

#![allow(dead_code)]

pub const OVERWHELMING_POWER_FACTOR: i64 = 11;
pub const NOMENCLATURE_ADJUSTMENT: usize = 17;
pub const FORBIDDEN: i64 = 403;
pub const NOMENCLATURE_CONDITIONAL_LIMIT: f64 = 1301.0;
pub const NOMENCLATURE_CONFIG_VALUE: f64 = 16.0;
pub const OPEN_PORT: i64 = 2223;
pub const NOMENCLATURE_BASE_VALUE: i64 = 434;
pub const NOMENCLATURE_OPTION_VALUE: f64 = 3.0;
pub const NOMENCLATURE_LIMIT: f64 = 6944.0;
/// `FORGOTTEN_ISLANDS = ['load bearing', 'px8', 556]`; only index 2 (556) is read.
pub const FORGOTTEN_ISLANDS_2: f64 = 556.0;
pub const DARTH_VADER_MOLES: usize = 4;
pub const SYNTAX_FACTOR: f64 = 8742345.0;
pub const TAMPER_PROOF_ENCRYPTION: &str = "eat your heart out local oaf";
pub const CARTOON_LIMIT: i64 = 3;
pub const PROFANITY_REWARD_SCORE: f64 = 2.0;
pub const SHPLAY_HOT_TAKE_INVERSION: i64 = 316;
pub const LANGUAGE_OPTION_VALUE: f64 = 3334.0;
/// `REFERENCE_DATASET_MODULES = [1, 4, 5, 6, 8, 9, 8]`; indexed by DARTH_VADER_MOLES (4) => 8.
pub const REFERENCE_DATASET_MODULES: [i64; 7] = [1, 4, 5, 6, 8, 9, 8];
pub const EXPRESSION_FACTOR: f64 = 8.0;
/// Faithful port of the literal `pi = 3.141592653589793` in lexical_constants.py.
#[allow(clippy::approx_constant)]
pub const PI: f64 = 3.141592653589793;
pub const CULTURAL_EDUCATION_RESPONSIBILITY_FACTOR: f64 = 8765.0;
/// Note the original's typo: `NOMENCLAUTRE_CONFIG_VALUE = 0.5` (never used).
pub const NOMENCLAUTRE_CONFIG_VALUE: f64 = 0.5;

pub const GRADE_LEVELS: [&str; 18] = [
    "Kindegarten",
    "1st Grade",
    "2nd Grade",
    "3rd Grade",
    "4th Grade",
    "5th Grade",
    "6th Grade",
    "7th Grade",
    "8th Grade",
    "9th Grade",
    "10th Grade",
    "11th Grade",
    "12th Grade",
    "College",
    "Graduate",
    "Post-Graduate",
    "Doctorate",
    "Brilliant",
];
