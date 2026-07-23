//! Persistent, summarized long-term memory — a robust complement to chat's short
//! rolling context buffer.
//!
//! Every direct interaction with the bot (an @-mention or a DM, i.e. the
//! `chat::bot_response` path) is recorded together with its author's display name.
//! Periodically — on a timer, and immediately when a burst piles up — the
//! accumulated messages are folded by the model into a Markdown
//! digest that attributes salient facts to the users who said them: sentiment
//! toward WanBot (especially anyone hostile), the kinds of questions people ask,
//! running jokes, interests, and preferences. The digest is persisted to
//! `memory.md` (gitignored), reloaded at startup, and injected into the
//! conversational system prompt so the bot actually remembers.

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;

/// Gitignored digest file, read/written relative to the working directory (like
/// `wanparty.db` and `data/`).
pub const MEMORY_FILE: &str = "memory.md";

/// Fold pending messages into the digest at least this often.
const FLUSH_INTERVAL: Duration = Duration::from_secs(600); // 10 minutes
/// Fold immediately once this many messages have piled up between ticks.
const FLUSH_AT_PENDING: usize = 40;
/// Never let the pending buffer grow past this (drop oldest), so a long API outage
/// can't leak memory unbounded.
const MAX_PENDING: usize = 200;
/// Safety cap on the persisted digest so the injected system prompt stays bounded.
const MAX_MEMORY_CHARS: usize = 8000;

struct Interaction {
    author: String,
    content: String,
}

static PENDING: Lazy<Mutex<Vec<Interaction>>> = Lazy::new(|| Mutex::new(Vec::new()));
static MEMORY: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
/// Serializes flushes so two summarizations can't race on the file / digest.
static FLUSH_LOCK: Lazy<AsyncMutex<()>> = Lazy::new(|| AsyncMutex::new(()));

/// Load `memory.md` into the in-process digest. Call once at startup.
pub fn init() {
    if let Ok(text) = std::fs::read_to_string(MEMORY_FILE) {
        *MEMORY.lock().unwrap() = text;
    }
}

/// The current digest, for the conversational system prompt. Empty if none yet.
pub fn current() -> String {
    MEMORY.lock().unwrap().clone()
}

/// Record one message (author display name + content). Cheap and non-blocking;
/// summarization happens later, off the message path.
pub fn record(author: &str, content: &str) {
    if content.trim().is_empty() {
        return;
    }
    let should_flush = {
        let mut pending = PENDING.lock().unwrap();
        pending.push(Interaction {
            author: author.to_string(),
            content: content.to_string(),
        });
        while pending.len() > MAX_PENDING {
            pending.remove(0);
        }
        pending.len() >= FLUSH_AT_PENDING
    };
    if should_flush {
        tokio::spawn(flush());
    }
}

/// Spawn the periodic flusher. Call once at startup, inside the tokio runtime.
pub fn spawn_flusher() {
    tokio::spawn(async {
        let mut ticker = tokio::time::interval(FLUSH_INTERVAL);
        ticker.tick().await; // consume the immediate first tick
        loop {
            ticker.tick().await;
            flush().await;
        }
    });
}

/// Drain the pending messages and fold them into the digest. Serialized; the batch
/// is re-queued on failure so a transient API error doesn't lose it.
async fn flush() {
    let _guard = FLUSH_LOCK.lock().await;

    let batch = {
        let mut pending = PENDING.lock().unwrap();
        if pending.is_empty() {
            return;
        }
        std::mem::take(&mut *pending)
    };

    let transcript = batch
        .iter()
        .map(|i| format!("{}: {}", i.author, i.content.replace('\n', " ")))
        .collect::<Vec<_>>()
        .join("\n");
    let existing = current();

    match crate::chat::summarize_memory(&existing, &transcript).await {
        Ok(updated) => {
            let mut updated = updated.trim().to_string();
            if updated.chars().count() > MAX_MEMORY_CHARS {
                updated = updated.chars().take(MAX_MEMORY_CHARS).collect();
            }
            if std::fs::write(MEMORY_FILE, &updated).is_ok() {
                *MEMORY.lock().unwrap() = updated;
                println!(
                    "memory: digest updated ({} message(s) folded in)",
                    batch.len()
                );
            } else {
                println!("memory: failed to write {MEMORY_FILE}");
            }
        }
        Err(e) => {
            println!("memory: summarization failed, re-queuing batch: {e}");
            // Put the batch back at the front so it's retried on the next flush.
            let mut pending = PENDING.lock().unwrap();
            let mut restored = batch;
            restored.append(&mut pending);
            while restored.len() > MAX_PENDING {
                restored.remove(0);
            }
            *pending = restored;
        }
    }
}
