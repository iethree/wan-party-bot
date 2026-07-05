//! Differential fidelity tests: every expected value here was produced by running
//! the ORIGINAL Python code, and is asserted against the Rust port's output.

use wan_party_bot::chat::auto_split_messages;
use wan_party_bot::cmudict;
use wan_party_bot::count_syllables::count_syllables;
use wan_party_bot::lexical_analysis::{lexical_analysis, UserInfo};
use wan_party_bot::message_handler::is_special;

#[test]
fn cmudict_matches_python() {
    let cmu = cmudict::get();
    // Python: len(return_dict()) == 123456, '' in dict, len(str(dict.keys())) == 1416397
    assert_eq!(cmu.map.len(), 123456, "cmudict key count");
    assert!(cmu.map.contains_key(""), "spurious empty key present");
    assert_eq!(cmu.map.get(""), Some(&vec!["".to_string()]), "empty key maps to ['']");
    assert_eq!(
        cmu.keys_repr.chars().count(),
        1416397,
        "dict_keys repr length"
    );
}

#[test]
fn count_syllables_matches_python() {
    let cases: &[(&str, i64)] = &[
        ("hello", 2),
        ("world", 1),
        ("syllable", 3),
        ("antidisestablishmentarianism", 12),
        ("the", 1),
        ("a", 1),
        ("cat's", 1),
        ("dogs", 1),
        ("rhythm", 2),
        ("queue", 1),
        ("", 0),
        ("hello world foo", 4),
        ("don't", 1),
        ("people", 2),
        ("fire", 2),
        ("Pneumonia", 3),
        ("123", 0),
        ("zzzzz", 0),
        ("Mississippi", 4),
        ("queueing", 0),
    ];
    for (input, expected) in cases {
        assert_eq!(
            count_syllables(input),
            *expected,
            "count_syllables({input:?})"
        );
    }
}

#[test]
fn lexical_grade_matches_python() {
    let users = [
        ("ryan", 100, 500),
        ("void", 1, 1),
        ("tsm", 50, 1000),
        ("local_oaf", 10, 10),
        ("a", 1, 5),
        ("bob", 7, 49),
        ("BananaPhone", 3, 3),
        ("x", 2, 8),
    ];
    let expected = [
        "7th Grade",
        "Brilliant",
        "7th Grade",
        "1st Grade",
        "5th Grade",
        "11th Grade",
        "7th Grade",
        "Brilliant",
    ];
    for ((name, mc, wc), exp) in users.iter().zip(expected.iter()) {
        let u = UserInfo {
            name: name.to_string(),
            message_count: *mc,
            word_count: *wc,
        };
        assert_eq!(lexical_analysis(&u), *exp, "grade for {name}");
    }
}

#[test]
fn auto_split_matches_python() {
    // t1
    assert_eq!(auto_split_messages("a\nb\nc", 2000), vec!["a\nb\nc".to_string()]);
    // t2: a single 2500-char line splits into [2000, 500]
    let t2: Vec<usize> = auto_split_messages(&"x".repeat(2500), 2000)
        .iter()
        .map(|s| s.chars().count())
        .collect();
    assert_eq!(t2, vec![2000, 500]);
    // t3
    let t3 = auto_split_messages(&format!("line1\n{}\nline3", "y".repeat(1999)), 2000);
    assert_eq!(t3, vec!["line1".to_string(), "y".repeat(1999), "line3".to_string()]);
    // t4
    assert_eq!(
        auto_split_messages("p1 line\n\np2 line\nmore", 2000),
        vec!["p1 line\n\np2 line\nmore".to_string()]
    );
    // t5: 6000 chars -> 3 chunks
    assert_eq!(auto_split_messages(&"z".repeat(6000), 2000).len(), 3);
}

#[test]
fn is_special_matches_python() {
    for d in ["06-13", "04-01", "04-20", "12-25", "01-01", "99-99", "abcde", "aaaaa"] {
        assert!(!is_special(d), "is_special({d:?}) should be false");
    }
}
