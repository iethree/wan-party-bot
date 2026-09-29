//! TypeSafe's Jev — a "System One" model that answers bounded questions with a
//! calibrated probability instead of prose.
//!
//! Where [`crate::chat`] asks Claude to *write* something, Jev just decides. We use
//! it for the jobs a chat model is overkill for: working out whether someone
//! actually asked a yes/no question and, if they did, what the answer is; and
//! picking an emoji to react to a message with.
//!
//! The yes/no judgments ride in a single request. `answer_yes` is speculative — it is
//! asked unconditionally and simply ignored when `is_yes_no` comes back low —
//! which is the documented way to avoid a second round trip:
//! <https://docs.typesafe.ai/patterns/fan-out>.
//!
//! Reads `JEV_API_KEY` from the environment. (The TypeSafe SDKs default to
//! `TYPESAFE_API_KEY`; we call the REST endpoint directly, so the name is ours.)

use crate::top_emoji::TOP_EMOJI;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::time::Duration;

const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
const MODEL: &str = "jev-latest";

/// How sure Jev has to be that this really is a yes/no question before we answer
/// it ourselves instead of handing the message to Claude. Deliberately high: a
/// wrong "yes" in place of a real conversation is far more annoying than a
/// conversational reply to something that could have been answered yes/no.
const IS_YES_NO_THRESHOLD: f64 = 0.8;

/// How much of the probability the top emoji needs before we react with it. Below
/// this, no reaction at all. Deliberately low: near-synonyms (🎂/🎉/🥳) split the
/// probability, so even an obvious pick often lands around 0.5 — this only filters
/// out messages where Jev has no real idea (a flat spread across dozens of emoji).
const REACTION_THRESHOLD: f64 = 0.1;

/// A routing call blocks every reply behind it, and every failure falls through to
/// Claude anyway, so don't wait around.
static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

/// The two probabilities behind one routing decision.
pub struct YesNoVerdict {
    /// Probability that the message is a yes/no question.
    pub is_yes_no: f64,
    /// Probability that the answer to it is "yes". Only meaningful when
    /// [`YesNoVerdict::is_yes_no_question`] holds — it's asked speculatively.
    pub answer_yes: f64,
}

impl YesNoVerdict {
    /// Whether this is confidently a yes/no question, i.e. ours to answer.
    pub fn is_yes_no_question(&self) -> bool {
        self.is_yes_no >= IS_YES_NO_THRESHOLD
    }

    /// The reply, hedged to match how sure Jev actually is. A noul is a
    /// probability, not a coin flip, so 0.55 shouldn't read the same as 0.99.
    pub fn phrase_answer(&self) -> &'static str {
        match self.answer_yes {
            p if p >= 0.95 => "yes",
            p if p >= 0.80 => "yeah, probably",
            p if p >= 0.60 => "leaning yes",
            p if p > 0.40 => "could honestly go either way 🤷",
            p if p > 0.20 => "leaning no",
            p if p > 0.05 => "probably not",
            _ => "no",
        }
    }
}

/// Ask Jev both questions about `question` in one request.
///
/// `replying_to` is the content of the message being replied to, when there is one
/// — "is that true?" can't be answered without it. `memory` is the long-term digest
/// ([`crate::memory::current`]), the same context Claude gets, so a question that
/// turns on who someone is doesn't get answered cold; pass `""` for none.
pub async fn yes_no_verdict(
    question: &str,
    replying_to: Option<&str>,
    memory: &str,
) -> Result<YesNoVerdict, String> {
    // Named fields rather than one blob, so the questions can point at `message`
    // and leave the rest as context: https://docs.typesafe.ai/concepts/state
    // Jev ingests the state once and evaluates both questions against it, so the
    // digest is paid for once per request, not once per question.
    let mut state = json!({ "message": question });
    if let Some(quoted) = replying_to {
        state["replying_to"] = json!(quoted);
    }
    if !memory.trim().is_empty() {
        state["memory"] = json!(memory);
    }

    let body = json!({
        "model": MODEL,
        "state": state,
        "questions": {
            "is_yes_no": {
                "type": "noul",
                "instructions": "Is `message` a yes/no question?",
                "criteria": {
                    "true": "A closed question whose natural answer is just yes or no — \"is it raining?\", \"should I buy this?\", \"do you like pizza?\", \"am I wrong?\", \"is that true?\"",
                    "false": "Anything else: an open question (who, what, when, where, why, how, or one asking for a list, an explanation, or an opinion at length), an instruction to do or write something, or a statement that isn't a question at all"
                }
            },
            "answer_yes": {
                "type": "noul",
                "instructions": "Assuming `message` is a yes/no question, is the correct answer to it yes? `memory`, when present, is what the bot remembers about the people in this Discord server, in sections keyed by user id — lean on it when the question turns on who someone is, what they like, or what they've said before, and ignore it otherwise.",
                "criteria": {
                    "true": "Yes — the thing being asked about is true, correct, advisable, or the case",
                    "false": "No — the thing being asked about is false, incorrect, inadvisable, or not the case"
                }
            }
        }
    });

    let value = ask(&body).await?;

    Ok(YesNoVerdict {
        is_yes_no: noul(&value, "is_yes_no")?,
        answer_yes: noul(&value, "answer_yes")?,
    })
}

/// Jev's pick of a reaction for a message: the most likely emoji and how much of
/// the probability it got.
pub struct ReactionPick {
    pub emoji: &'static str,
    pub probability: f64,
}

impl ReactionPick {
    /// Whether Jev is sure enough of this emoji to actually react with it.
    pub fn is_confident(&self) -> bool {
        self.probability >= REACTION_THRESHOLD
    }
}

/// Ask Jev which of [`TOP_EMOJI`] best fits as a reaction to `message`.
///
/// One Choice over all 200 options (the limit is 255), rather than a shortlist:
/// it's a few tokens per option and the model can't pick what it isn't shown.
/// Near-synonyms (😂/🤣, the dozen hearts) do split the probability between them,
/// which is why [`REACTION_THRESHOLD`] is so low.
pub async fn reaction(message: &str) -> Result<ReactionPick, String> {
    let criteria: serde_json::Map<String, Value> = TOP_EMOJI
        .iter()
        .map(|(name, emoji)| (name.to_string(), json!(emoji)))
        .collect();

    let body = json!({
        "model": MODEL,
        "state": { "message": message },
        "questions": {
            "reaction": {
                "type": "choice",
                "instructions": "A friend in a casual gaming Discord server is reacting to `message` with a single emoji. Which emoji would they most naturally react with?",
                "criteria": criteria
            }
        }
    });

    let value = ask(&body).await?;
    let probabilities = value
        .get("answers")
        .and_then(|a| a.get("reaction"))
        .and_then(|a| a.get("probabilities"))
        .and_then(|p| p.as_object())
        .ok_or_else(|| format!("no choice answer for 'reaction' in {value}"))?;

    // Take the max ourselves rather than trusting `choice`, so the emoji and the
    // probability it's judged on can't disagree.
    TOP_EMOJI
        .iter()
        .filter_map(|(name, emoji)| {
            let p = probabilities.get(*name)?.as_f64()?;
            Some(ReactionPick { emoji, probability: p })
        })
        .max_by(|a, b| a.probability.total_cmp(&b.probability))
        .ok_or_else(|| format!("no known emoji in {value}"))
}

/// POST one System One request and hand back the parsed response.
async fn ask(body: &Value) -> Result<Value, String> {
    let api_key = std::env::var("JEV_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err("JEV_API_KEY is not set".to_string());
    }

    let resp = HTTP
        .post(ENDPOINT)
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("typesafe error {status}: {text}"));
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// Pull one `answers.<id>.noul` probability out of the response.
fn noul(response: &Value, id: &str) -> Result<f64, String> {
    response
        .get("answers")
        .and_then(|a| a.get(id))
        .and_then(|a| a.get("noul"))
        .and_then(|n| n.as_f64())
        .ok_or_else(|| format!("no noul answer for '{id}' in {response}"))
}
