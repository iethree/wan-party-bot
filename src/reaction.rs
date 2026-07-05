//! Port of `reaction.py` — the `Reaction` / `MatchingReaction` abstraction.
//!
//! The live reaction engine in [`crate::message_handler`] inlines its checks, so
//! this module mirrors the original classes (and their tests) for completeness.
//!
//! Note: `test_reaction.py` calls `reaction.apply_to(msg)` without `await`, so the
//! coroutine never runs and the asserted reactions are never actually applied —
//! the Python tests as written would fail / warn. The Rust tests below implement
//! the clearly-intended behavior and pass.

#![allow(dead_code)]

/// `listify(maybe_list)` — wrap a scalar into a single-element vector. (In Rust the
/// list-or-scalar polymorphism is handled by the typed constructors below.)
pub fn listify<T>(value: T) -> Vec<T> {
    vec![value]
}

pub struct Reaction {
    pub keywords: Vec<String>,
    pub reactions: Vec<String>,
}

impl Reaction {
    pub fn new(keywords: Vec<&str>, reactions: Vec<&str>) -> Self {
        Self {
            keywords: keywords.into_iter().map(String::from).collect(),
            reactions: reactions.into_iter().map(String::from).collect(),
        }
    }

    /// `any(kw in content for kw in self.keywords)`.
    pub fn matches(&self, content: &str) -> bool {
        self.keywords.iter().any(|kw| content.contains(kw))
    }

    /// `apply_to(message)` — collects the reactions onto the target (the live code
    /// calls `message.add_reaction`; tests use a Vec collector).
    pub fn apply_to(&self, target: &mut Vec<String>) {
        for reaction in &self.reactions {
            println!("Adding {reaction}!");
            target.push(reaction.clone());
        }
    }
}

pub struct MatchingReaction {
    matcher: Box<dyn Fn(&str) -> bool + Send + Sync>,
    pub reactions: Vec<String>,
}

impl MatchingReaction {
    pub fn new(
        matcher: Box<dyn Fn(&str) -> bool + Send + Sync>,
        reactions: Vec<&str>,
    ) -> Self {
        Self {
            matcher,
            reactions: reactions.into_iter().map(String::from).collect(),
        }
    }

    pub fn matches(&self, content: &str) -> bool {
        (self.matcher)(content)
    }

    pub fn apply_to(&self, target: &mut Vec<String>) {
        for reaction in &self.reactions {
            println!("Adding {reaction}!");
            target.push(reaction.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_whole_dang_thing() {
        let mut reactions: Vec<String> = Vec::new();
        let reaction = Reaction::new(vec!["poop"], vec!["💩"]);
        if reaction.matches("poop") {
            reaction.apply_to(&mut reactions);
        }
        assert_eq!(vec!["💩".to_string()], reactions);
    }

    #[test]
    fn test_the_other_whole_dang_thing() {
        let mut reactions: Vec<String> = Vec::new();
        let reaction = MatchingReaction::new(Box::new(|c: &str| c.contains("poop")), vec!["💩"]);
        if reaction.matches("poop") {
            reaction.apply_to(&mut reactions);
        }
        assert_eq!(vec!["💩".to_string()], reactions);
    }
}
