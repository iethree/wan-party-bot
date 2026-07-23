//! WAN Party Discord Bot — a faithful Rust port of the original Python application.
//!
//! Every Python module is reproduced here, quirks and all. Where the Python relied
//! on `discord.py`, the Rust port uses `serenity`. Where it relied on the Anthropic
//! Python SDK, the port calls the Anthropic REST API directly with `reqwest`.
//!
//! Fidelity notes for the genuinely weird bits live next to the code that
//! reproduces them. The two most important global facts:
//!   * `main.py` has `tree.sync()` commented out, so NONE of the `commands.py`
//!     slash commands are ever registered with Discord. They are ported in
//!     [`commands`] for completeness but, exactly like the original, are never
//!     wired up. The only live command surface is the text-prefix handling in
//!     [`message_handler`]/[`main`] and the AI features in [`chat`].
//!   * All Python `open(path).read()` calls run in text mode, which performs
//!     universal-newline translation (\r\n -> \n). [`text_util::read_text`]
//!     reproduces that so CRLF data files (e.g. `data/dick.txt`) split identically.

/// Build-time version: the total number of commits on the branch
/// (`git rev-list --count HEAD`), baked in by `build.rs`. Falls back to `"unknown"`
/// when built outside a git checkout (e.g. from a source tarball).
pub const VERSION: &str = env!("BUILD_VERSION");

pub mod blacklist;
pub mod chat;
pub mod client;
pub mod cmudict;
pub mod commands;
pub mod count_syllables;
pub mod db;
pub mod discord_util;
pub mod error_messages;
pub mod giphy;
pub mod jokes;
pub mod leaderboards;
pub mod lexical_analysis;
pub mod lexical_constants;
pub mod markov_haiku;
pub mod memory;
pub mod message_handler;
pub mod poll;
pub mod quote;
pub mod quotes;
pub mod reaction;
pub mod rhymes;
pub mod sing;
pub mod text_util;
pub mod thinking;

/// Global, process-wide state captured at gateway READY, mirroring how the Python
/// code reaches for `client.user`. `main.py` checks `str(client.user.id) in content`
/// (a substring test against the raw decimal id) and `chat.py` uses
/// `client.user.mention`.
pub mod botself {
    use std::sync::OnceLock;

    static BOT_ID: OnceLock<u64> = OnceLock::new();

    pub fn set_bot_id(id: u64) {
        let _ = BOT_ID.set(id);
    }

    pub fn bot_id() -> u64 {
        *BOT_ID.get().unwrap_or(&0)
    }

    /// The raw decimal id as a string, matching `str(client.user.id)`.
    pub fn bot_id_str() -> String {
        bot_id().to_string()
    }

    /// `client.user.mention` => `<@id>`.
    pub fn bot_mention() -> String {
        format!("<@{}>", bot_id())
    }
}
