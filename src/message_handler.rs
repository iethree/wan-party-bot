//! Port of `message_handler.py` — the reaction engine and the `#!`/`##`/"what
//! think" responder invoked from `on_message`.
//!
//! SECURITY NOTE (preserved behavior, not introduced here): the original bot
//! treats a message beginning with `#!` as a shell command and one beginning with
//! `##` as Python code to execute, returning the captured output. These are
//! arbitrary-code-execution backdoors in the original application. They are
//! reproduced faithfully per the request to preserve all functionality:
//!   * `#!` runs the remaining whitespace-separated tokens as a subprocess
//!     (no shell), exactly like `subprocess.run(list)`.
//!   * `##` cannot be reproduced exactly — Rust has no embedded Python with the
//!     bot's globals — so it shells out to `python3 -c <content>`, which matches
//!     the common case (the first `##` line is a Python comment).

use crate::botself;
use crate::chat;
use crate::discord_util::{self, get_emoji_reaction, unicode};
use crate::quotes;
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use serenity::all::{Context, Message, ReactionType};

fn sometimes(chance: f64) -> bool {
    rand::thread_rng().gen::<f64>() < chance
}

/// `did_u_say_wanbot`.
static WANBOT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bw[\w*]{0,3}n.{0,2}[8b][\w*]{0,3}[ty]\b").unwrap());

/// The "thedeck" matcher regex.
static DECK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"d[eéèë*3!]ck|d[oóòö*0!]nk|g[*\w]{0,2}b[*\w]{0,2}\s*g[*\w]{0,5}r").unwrap()
});

/// The magic words for the 🙄 reaction.
const MAGIC_WORDS: [&str; 15] = [
    "opinion",
    "take",
    "ps5",
    "valve",
    "steam deck",
    "harry potter",
    "dune",
    "star wars",
    "wow",
    "factorio",
    "the last of us",
    "overwatch",
    "facts",
    "objective",
    "correct",
];

/// The "vοid" sentinel — note the Greek small letter omicron (U+03BF), not a
/// Latin 'o'. The original searches author usernames for this exact substring.
const VOID: &str = "v\u{03bf}id";

fn today_mmdd() -> String {
    chrono::Local::now().format("%m-%d").to_string()
}

/// `is_special = 614049 == sum(ord(s) * 10**o for (s,o) in zip(today, range(5)))`.
pub fn is_special(today: &str) -> bool {
    let mut total: i64 = 0;
    for (o, s) in today.chars().take(5).enumerate() {
        total += (s as i64) * 10i64.pow(o as u32);
    }
    total == 614049
}

async fn react_all(ctx: &Context, msg: &Message, reactions: &[ReactionType]) {
    for r in reactions {
        println!("Adding {r}!");
        let _ = msg.react(&ctx.http, r.clone()).await;
    }
}

/// `respond_to(client, message)`. Returns `Some(text)` only for the `#!`, `##`,
/// and "what think" branches; otherwise applies reactions/responses and returns
/// `None`.
pub async fn respond_to(ctx: &Context, message: &Message) -> Option<String> {
    let raw = &message.content;

    // `#!` — run the remaining tokens as a subprocess (no shell).
    if raw.starts_with("#!") {
        let parts: Vec<&str> = raw.split_whitespace().collect();
        if parts.len() < 2 {
            // subprocess.run([]) raises in Python -> handler aborts; nothing sent.
            return None;
        }
        let cmd = parts[1];
        let args = &parts[2..];
        match std::process::Command::new(cmd).args(args).output() {
            Ok(out) => {
                let mut bytes = out.stdout;
                bytes.extend_from_slice(&out.stderr);
                return Some(String::from_utf8_lossy(&bytes).into_owned());
            }
            Err(_) => return None, // FileNotFoundError -> handler aborts; nothing sent.
        }
    }

    // `##` — execute as Python (see security note above).
    if raw.starts_with("##") {
        match std::process::Command::new("python3")
            .arg("-c")
            .arg(raw)
            .output()
        {
            Ok(out) => {
                let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
                s.push_str(&String::from_utf8_lossy(&out.stderr));
                return Some(s);
            }
            Err(e) => return Some(e.to_string()),
        }
    }

    let content = raw.to_lowercase();

    // "what think" — quote a recent message from a "vοid"-named author.
    let bot_id_str = botself::bot_id_str();
    if content.contains(&bot_id_str) && content.contains("what") && content.contains("think") {
        let history = discord_util::channel_history(ctx, message.channel_id, Some(64)).await;
        for m in &history {
            if m.id == message.id {
                continue;
            }
            if !m.author.name.to_lowercase().contains(VOID) {
                continue;
            }
            return Some(m.content.clone());
        }
        return Some("hmmm".to_string());
    }

    let today = today_mmdd();
    let special = is_special(&today);
    let op = discord_util::display_name(message).to_lowercase();

    // Guild emoji map (built once from cache, mirroring `guild.emojis`).
    let emoji_map = match message.guild_id {
        Some(gid) => discord_util::guild_emoji_map(ctx, gid),
        None => std::collections::HashMap::new(),
    };
    let channel_name = discord_util::channel_name(ctx, message.channel_id).await;

    // ---- STATIC_REACTIONS (in order) ----
    if content.contains("poop") {
        react_all(ctx, message, &[unicode("💩")]).await;
    }
    if content.contains("drg") || content.contains("dwarf") {
        // rock and stone
        react_all(ctx, message, &[unicode("🪨"), unicode("🥌")]).await;
    }
    if content.contains("ps5") {
        react_all(ctx, message, &[unicode("👎")]).await;
    }
    if content.contains("dongle") {
        react_all(ctx, message, &[unicode("🍆")]).await;
    }
    if content.contains("how you doin bot?") {
        react_all(ctx, message, &[unicode("👍")]).await;
    }
    if sometimes(0.1) && MAGIC_WORDS.iter().any(|w| content.contains(w)) {
        react_all(ctx, message, &[unicode("🙄")]).await;
    }
    if WANBOT_RE.is_match(&content) {
        react_all(ctx, message, &[unicode("💁‍♀️")]).await;
    }

    // ---- dynamic guild-emoji reactions (in order) ----
    if content.contains("the way") {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "mando")]).await;
    }
    if content.contains("meta") {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "meta")]).await;
    }
    if content.contains("texas") {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "happytexas")]).await;
    }
    if content.contains("star wars") && channel_name.as_deref() != Some("star-wars") {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "stormtrooper")]).await;
    }
    if DECK_RE.is_match(&content) {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "thedeck")]).await;
    }
    if op.contains("ryan") && (content.contains("wow") || content.contains("warcraft")) {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "wow")]).await;
    }
    if op.contains("tsm") && sometimes(0.03) {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "neato")]).await;
    }
    if op.contains("local_oaf") && sometimes(0.01) {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "elon")]).await;
    }
    // The "elTa" + 13 cipher resolves to "ryan"; gated on is_special.
    if op.contains("ryan") && special {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "wow")]).await;
    }
    if op.contains("shplay") && today == "04-20" {
        react_all(ctx, message, &[get_emoji_reaction(&emoji_map, "420shplaybday")]).await;
    }

    // ---- responses (string triggers) ----
    if content.contains("yoda") {
        let _ = message
            .channel_id
            .say(&ctx.http, quotes::get_yoda_quote())
            .await;
    }
    if content.contains("trombone") {
        let _ = message
            .channel_id
            .say(
                &ctx.http,
                "https://twitter.com/JacobDJAtkinson/status/1572449169666703360",
            )
            .await;
    }

    if sometimes(0.07) {
        chat::appropriate_reaction(ctx, message).await;
    }

    None
}
