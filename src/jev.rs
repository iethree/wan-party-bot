//! TypeSafe's Jev — a "System One" model that answers bounded questions with a
//! calibrated probability instead of prose.
//!
//! Where [`crate::chat`] asks Claude to *write* something, Jev just decides. We use
//! it for the one job a chat model is overkill for: working out whether someone
//! actually asked a yes/no question and, if they did, what the answer is.
//!
//! Both judgments ride in a single request. `answer_yes` is speculative — it is
//! asked unconditionally and simply ignored when `is_yes_no` comes back low —
//! which is the documented way to avoid a second round trip:
//! <https://docs.typesafe.ai/patterns/fan-out>.
//!
//! Reads `JEV_API_KEY` from the environment. (The TypeSafe SDKs default to
//! `TYPESAFE_API_KEY`; we call the REST endpoint directly, so the name is ours.)

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

/// Ask Jev both questions about `question` in one request. `replying_to` is the
/// content of the message being replied to, when there is one — "is that true?"
/// can't be answered without it.
pub async fn yes_no_verdict(
    question: &str,
    replying_to: Option<&str>,
) -> Result<YesNoVerdict, String> {
    let api_key = std::env::var("JEV_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err("JEV_API_KEY is not set".to_string());
    }

    // Named fields rather than one blob, so the questions can point at `message`
    // and leave `replying_to` as context: https://docs.typesafe.ai/concepts/state
    let mut state = json!({ "message": question });
    if let Some(quoted) = replying_to {
        state["replying_to"] = json!(quoted);
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
                "instructions": "Assuming `message` is a yes/no question, is the correct answer to it yes?",
                "criteria": {
                    "true": "Yes — the thing being asked about is true, correct, advisable, or the case",
                    "false": "No — the thing being asked about is false, incorrect, inadvisable, or not the case"
                }
            }
        }
    });

    let resp = HTTP
        .post(ENDPOINT)
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("typesafe error {status}: {text}"));
    }
    let value: Value = resp.json().await.map_err(|e| e.to_string())?;

    Ok(YesNoVerdict {
        is_yes_no: noul(&value, "is_yes_no")?,
        answer_yes: noul(&value, "answer_yes")?,
    })
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
