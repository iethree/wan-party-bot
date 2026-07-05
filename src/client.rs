//! Port of `client.py` — gateway intents.
//!
//! Python: `Intents.default()` (all non-privileged) with `messages`,
//! `message_content`, and `reactions` enabled. `message_content` is privileged;
//! the others are already covered by the non-privileged set.

use serenity::all::GatewayIntents;

pub fn intents() -> GatewayIntents {
    GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT
}
