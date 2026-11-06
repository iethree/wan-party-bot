//! How a command was run, so a feature can report back the same way whether it
//! was typed as a text prefix (`/comeback` as a reply) or run as a Discord
//! application command (slash command, right-click menu, or modal form).
//!
//! The feature's real output — a reply to the target message, an embed — goes to
//! the channel either way. Only the side-channel status (the 🤷 / 💰 / ✅
//! reactions) differs: a text command gets reactions on the command message; an
//! application command has no message to react to, so the status is written to
//! its private "thinking…" response instead.

use crate::discord_util;
use serenity::all::{Context, EditInteractionResponse, Message};
use serenity::builder::Builder;
use std::sync::Mutex;

pub enum Invoker<'a> {
    /// A text-prefix command. Status is reactions on the command message.
    Message(&'a Message),
    /// An application command already acknowledged with a deferred ephemeral
    /// response. Status edits that private response; [`Invoker::finish`] deletes
    /// it when nothing was reported, so a success leaves no trace.
    Interaction {
        token: String,
        notes: Mutex<Vec<String>>,
    },
}

impl<'a> Invoker<'a> {
    /// An application command whose interaction `token` has been deferred
    /// ephemerally.
    pub fn interaction(token: &str) -> Self {
        Invoker::Interaction {
            token: token.to_string(),
            notes: Mutex::new(Vec::new()),
        }
    }

    /// The command message, for a text-prefix command.
    pub fn message(&self) -> Option<&'a Message> {
        match self {
            Invoker::Message(m) => Some(m),
            Invoker::Interaction { .. } => None,
        }
    }

    /// Report a status emoji.
    pub async fn react(&self, ctx: &Context, emoji: &str) {
        match self {
            Invoker::Message(m) => {
                let _ = m.react(&ctx.http, discord_util::unicode(emoji)).await;
            }
            Invoker::Interaction { .. } => self.note(ctx, emoji).await,
        }
    }

    /// Report a line of text. A text-prefix command replies to the command
    /// message with it.
    pub async fn say(&self, ctx: &Context, text: &str) {
        match self {
            Invoker::Message(m) => {
                let _ = m.reply_ping(&ctx.http, text).await;
            }
            Invoker::Interaction { .. } => self.note(ctx, text).await,
        }
    }

    async fn note(&self, ctx: &Context, text: &str) {
        let Invoker::Interaction { token, notes } = self else {
            return;
        };
        let content = {
            let mut notes = notes.lock().unwrap_or_else(|p| p.into_inner());
            notes.push(text.to_string());
            notes.join(" ")
        };
        let edit = EditInteractionResponse::new().content(content);
        if let Err(e) = edit.execute(&ctx.http, token).await {
            println!("error editing interaction response: {e}");
        }
    }

    /// Call when the command is done. Deletes an application command's private
    /// "thinking…" response if nothing was reported on it.
    pub async fn finish(&self, ctx: &Context) {
        let Invoker::Interaction { token, notes } = self else {
            return;
        };
        let empty = notes.lock().map(|n| n.is_empty()).unwrap_or(false);
        if empty {
            let _ = ctx.http.delete_original_interaction_response(token).await;
        }
    }
}
