//! Serenity helpers that reproduce the bits of `discord.py` the bot relies on:
//! `raw_mentions`, `display_name`, `channel.name`, guild-emoji lookup, and history
//! pagination.

use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{
    Channel, ChannelId, Context, Emoji, GetMessages, GuildId, Message, MessageId, ReactionType,
};
use std::collections::HashMap;

static MENTION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<@!?(\d+)>").unwrap());

/// Equivalent of `message.raw_mentions` — user ids parsed from `<@id>` / `<@!id>`.
pub fn raw_mentions(content: &str) -> Vec<u64> {
    MENTION_RE
        .captures_iter(content)
        .filter_map(|c| c.get(1).and_then(|m| m.as_str().parse::<u64>().ok()))
        .collect()
}

/// `message.author.display_name`: guild nick, else global name, else username.
pub fn display_name(msg: &Message) -> String {
    if let Some(member) = &msg.member {
        if let Some(nick) = &member.nick {
            return nick.clone();
        }
    }
    if let Some(global) = &msg.author.global_name {
        return global.clone();
    }
    msg.author.name.clone()
}

/// `message.channel.name` (None for DMs / non-named channels).
///
/// Python reads `message.channel.name` synchronously from the gateway cache, so it
/// always sees the real name. We read the cache first and only fall back to HTTP,
/// so a transient API hiccup can't silently turn a named (e.g. blacklisted) channel
/// into "no name".
pub async fn channel_name(ctx: &Context, channel_id: ChannelId) -> Option<String> {
    // `Cache::channel` is deprecated but is the only cache lookup keyed solely by
    // channel id (we don't always know the guild here); it still reads the cached
    // GuildChannel, which is exactly discord.py's synchronous `channel.name`.
    #[allow(deprecated)]
    if let Some(gc) = ctx.cache.channel(channel_id) {
        return Some(gc.name.clone());
    }
    match channel_id.to_channel(ctx).await {
        Ok(Channel::Guild(gc)) => Some(gc.name),
        _ => None,
    }
}

/// Build a `name -> Emoji` map from the cached guild, mirroring `guild.emojis`
/// (the Python code reads the cache, not the API).
pub fn guild_emoji_map(ctx: &Context, guild_id: GuildId) -> HashMap<String, Emoji> {
    if let Some(guild) = ctx.cache.guild(guild_id) {
        guild
            .emojis
            .values()
            .map(|e| (e.name.to_string(), e.clone()))
            .collect()
    } else {
        HashMap::new()
    }
}

pub fn emoji_to_reaction(e: &Emoji) -> ReactionType {
    ReactionType::Custom {
        animated: e.animated,
        id: e.id,
        name: Some(e.name.to_string()),
    }
}

/// `get_emoji(guild, name)` — the custom emoji if present, else the 🙃 fallback
/// (the original returns "🙃" on error and effectively never reacts when the emoji
/// is missing; we collapse both to the visible fallback).
pub fn get_emoji_reaction(map: &HashMap<String, Emoji>, name: &str) -> ReactionType {
    match map.get(name) {
        Some(e) => emoji_to_reaction(e),
        None => ReactionType::Unicode("🙃".to_string()),
    }
}

pub fn unicode(s: &str) -> ReactionType {
    ReactionType::Unicode(s.to_string())
}

/// `channel.history(limit=N)` — newest first, paginated in batches of 100.
/// `limit = None` fetches everything.
pub async fn channel_history(
    ctx: &Context,
    channel_id: ChannelId,
    limit: Option<usize>,
) -> Vec<Message> {
    let mut out: Vec<Message> = Vec::new();
    let mut before: Option<MessageId> = None;
    loop {
        let want: u8 = match limit {
            Some(l) => {
                let remaining = l.saturating_sub(out.len());
                if remaining == 0 {
                    break;
                }
                remaining.min(100) as u8
            }
            None => 100,
        };
        let mut builder = GetMessages::new().limit(want);
        if let Some(b) = before {
            builder = builder.before(b);
        }
        let batch = match channel_id.messages(&ctx.http, builder).await {
            Ok(b) => b,
            Err(_) => break,
        };
        if batch.is_empty() {
            break;
        }
        before = Some(batch.last().unwrap().id);
        let batch_len = batch.len();
        out.extend(batch);
        if batch_len < want as usize {
            break;
        }
    }
    out
}

const DISCORD_EPOCH_MS: i64 = 1420070400000;

/// `channel.history(limit=None, after=<time>)` — everything created after the
/// given unix-millisecond timestamp.
pub async fn channel_history_after(
    ctx: &Context,
    channel_id: ChannelId,
    after_ms: i64,
) -> Vec<Message> {
    let start = (((after_ms - DISCORD_EPOCH_MS).max(0)) as u64) << 22;
    let mut after_id = MessageId::new(start.max(1));
    let mut out: Vec<Message> = Vec::new();
    loop {
        let builder = GetMessages::new().limit(100).after(after_id);
        let batch = match channel_id.messages(&ctx.http, builder).await {
            Ok(b) => b,
            Err(_) => break,
        };
        if batch.is_empty() {
            break;
        }
        let max_id = batch.iter().map(|m| m.id).max().unwrap();
        after_id = max_id;
        let len = batch.len();
        out.extend(batch);
        if len < 100 {
            break;
        }
    }
    // discord.py's `history(after=..., limit=None)` defaults to `oldest_first=True`,
    // yielding a clean oldest->newest stream. Discord's REST batches don't guarantee
    // that order, so sort by id (snowflakes are time-ordered) to match.
    out.sort_by_key(|m| m.id);
    out
}
