//! Port of `chat.py` — the Anthropic-backed features.
//!
//! The Python uses the Anthropic SDK (`anthropic.Anthropic()`); we call the same
//! REST endpoint (`POST /v1/messages`) directly with `reqwest`, reading
//! `ANTHROPIC_API_KEY` from the environment exactly as the SDK does. Model and
//! token limits are preserved verbatim.

use crate::blacklist::is_blacklisted_channel;
use crate::botself;
use crate::discord_util;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use serenity::all::{Context, Message};
use std::sync::Mutex;

const MODEL: &str = "claude-haiku-4-5";
const MAX_TOKENS: u32 = 2048;

static HTTP: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

fn today_mmdd() -> String {
    chrono::Local::now().format("%m-%d").to_string()
}

/// `_extract_text(response)` — the first text block, or "".
fn extract_text(response: &Value) -> String {
    if let Some(content) = response.get("content").and_then(|c| c.as_array()) {
        for block in content {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                return block
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
            }
        }
    }
    String::new()
}

/// `ai_client.messages.create(...)` followed by `_extract_text`. Returns Err on any
/// failure, mirroring the SDK raising (callers wrap in try/except).
async fn create_message(system: &str, messages: Vec<Value>) -> Result<String, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let body = json!({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        "system": system,
        "messages": messages,
    });
    let resp = HTTP
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("anthropic error {status}: {text}"));
    }
    let value: Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(extract_text(&value))
}

// ----------------------------------------------------------------------------
// Personalities & prompts
// ----------------------------------------------------------------------------

const PERSONALITIES: [&str; 11] = [
    "sarcastic wise-cracking stand up comedian",
    "irascible grumpy pirate who has run out of rum",
    "sassy, no-nonsense, tell-it-like-it-is friend",
    "horny, flirty, middle schooler",
    "paranoid conspiracy theorist",
    "pretentious, snobby, wine critic",
    "socially awkward genius",
    "1950s gangster who talks like a dame",
    "southern belle with a dirty mouth",
    "granola-eating, tree-hugging, nixon-hating 1960s hippie",
    "politician who will say anything to get elected",
];

fn get_random_personality() -> &'static str {
    let mut rng = rand::thread_rng();
    PERSONALITIES.choose(&mut rng).copied().unwrap()
}

const LIMIT_CONTEXT: &str = "responses absolutely cannot exceed 1800 characters";

/// `get_conditional_prompts()` — date-gated extra system instructions.
fn get_conditional_prompts() -> String {
    let today = today_mmdd();
    let mut text = String::new();
    if today == "04-01" {
        text.push_str(" You are obsessed with Rick Astley and make reference to 'Never Gonna Give You Up' in every conversation");
    }
    if today == "04-20" {
        text.push_str(" You make obnoxious references to 420, 'trees', and 'weed' in every conversation, tuned specifically to make someone's dad annoyed");
    }
    if today == "04-08" {
        text.push_str(" You're obsessed with April 8, the numbers 4, 8, 84, and 48, and make mention of these things whenever you can");
    }
    text
}

/// `get_personality()`.
fn get_personality() -> String {
    let standard = format!(
        "Your name is WanBot, aka {} and you are a helpful robot in a discord server with a keen sense of humor that does not inhibit your helpfulness ",
        botself::bot_mention()
    );
    standard + &get_conditional_prompts()
}

/// `get_comeback(msg)` — the non-AI fallback comebacks.
fn get_comeback(msg: &str) -> String {
    let comebacks = [
        "that's what she said 😏".to_string(),
        format!("your mom {msg}"),
        format!("no you {msg}"),
        "I know you are but what am I?".to_string(),
        "🙄".to_string(),
        "🤣".to_string(),
    ];
    let mut rng = rand::thread_rng();
    comebacks.choose(&mut rng).unwrap().clone()
}

async fn get_ai_comeback(msg: &str) -> Result<String, String> {
    let personality = get_random_personality();
    println!("answering as a {personality}");
    let text = create_message(
        personality,
        vec![json!({"role":"user","content": format!("write a short comeback to {msg}")})],
    )
    .await?;
    println!("{text}");
    Ok(text)
}

async fn get_tldr_response(msg: &str) -> Result<String, String> {
    let text = create_message(
        &get_personality(),
        vec![json!({"role":"user","content": format!("write an extremely short and mildly flippant tldr summary of: {msg}")})],
    )
    .await?;
    println!("{text}");
    Ok(text)
}

async fn get_ai_kindness(msg: &str) -> Result<String, String> {
    let system = format!(
        "Your name is WanBot and you are a kind, empathetic, sincere, tender-hearted therapist dealing with a fragile patient{}",
        get_conditional_prompts()
    );
    let text = create_message(
        &system,
        vec![json!({"role":"user","content": format!("write a short bit of kind encouragement in response to {msg}")})],
    )
    .await?;
    println!("{text}");
    Ok(text)
}

async fn get_ai_recap(username: &str, messages_text: &str) -> Result<String, String> {
    let system_prompt = "You are a peppy, energetic AI assistant that generates 'Year in Review' style recaps, similar to Spotify Wrapped or big tech annual summaries. Your tone should be enthusiastic, using emojis and corporate-friendly but fun language. You're aware that these recaps are kind of annoying, and you're subtly ironic about the whole thing.";
    let user_prompt = format!(
        "Here is a collection of discord messages from user '{username}' over the past year. Please generate a very short and snappy recap of what they have been talking about. Highlight key themes, recurring jokes, or specific interests. The recap MUST be less than 500 words. \n\nMessages:\n{messages_text}"
    );
    let text = create_message(
        system_prompt,
        vec![json!({"role":"user","content": user_prompt})],
    )
    .await?;
    println!("{text}");
    Ok(text)
}

async fn get_person_response(personality: &str, msg: &str) -> Result<String, String> {
    let text = create_message(
        &format!("You are {personality}"),
        vec![json!({"role":"user","content": format!("respond to someone saying {msg}")})],
    )
    .await?;
    println!("{text}");
    Ok(text)
}

// ----------------------------------------------------------------------------
// Shared context buffer (global, size 10), exactly like the Python module global.
// ----------------------------------------------------------------------------

const CONTEXT_BUFFER_SIZE: usize = 10;

#[derive(Clone)]
struct ContextMsg {
    role: String,
    content: String,
}

static CONTEXT_BUFFER: Lazy<Mutex<Vec<ContextMsg>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn add_to_context(role: &str, msg: &str) {
    let mut buf = CONTEXT_BUFFER.lock().unwrap();
    buf.push(ContextMsg {
        role: role.to_string(),
        content: msg.to_string(),
    });
    if buf.len() > CONTEXT_BUFFER_SIZE {
        buf.remove(0);
    }
}

// ----------------------------------------------------------------------------
// Message splitting
// ----------------------------------------------------------------------------

/// `auto_split_messages(text, limit=2000)`. Lengths are character counts (Python
/// `len()` over `str`).
pub fn auto_split_messages(text: &str, limit: usize) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let mut current_chunk = String::new();

    for paragraph in text.split('\n') {
        let cclen = current_chunk.chars().count();
        let plen = paragraph.chars().count();
        if cclen + plen + 1 > limit {
            if !current_chunk.is_empty() {
                messages.push(current_chunk.trim().to_string());
                current_chunk.clear();
            }
            if plen > limit {
                let chars: Vec<char> = paragraph.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + limit).min(chars.len());
                    messages.push(chars[i..end].iter().collect());
                    i += limit;
                }
            } else {
                current_chunk = format!("{paragraph}\n");
            }
        } else {
            current_chunk.push_str(paragraph);
            current_chunk.push('\n');
        }
    }

    if !current_chunk.is_empty() {
        messages.push(current_chunk.trim().to_string());
    }
    messages
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

/// `get_quoted_msg(message)`.
pub async fn get_quoted_msg(ctx: &Context, msg: &Message) -> Option<Message> {
    let mref = msg.message_reference.as_ref()?;
    let mid = mref.message_id?;
    match msg.channel_id.message(&ctx.http, mid).await {
        Ok(m) => Some(m),
        Err(e) => {
            println!("error getting quoted message");
            println!("{e}");
            None
        }
    }
}

/// Wrapper around `is_blacklisted_channel(message.channel.name)`.
async fn channel_blacklisted(ctx: &Context, msg: &Message) -> bool {
    let name = discord_util::channel_name(ctx, msg.channel_id).await;
    is_blacklisted_channel(name.as_deref())
}

async fn react(ctx: &Context, msg: &Message, emoji: &str) {
    let _ = msg
        .react(&ctx.http, discord_util::unicode(emoji))
        .await;
}

// ----------------------------------------------------------------------------
// Public async entrypoints (called from main.py's on_message)
// ----------------------------------------------------------------------------

/// `get_bot_response(message)` — the conversational reply, with shared context.
async fn get_bot_response(ctx: &Context, message: &Message) -> Result<String, String> {
    let msg = message.content.clone();

    let quoted = get_quoted_msg(ctx, message).await;
    let mut user_content = msg.clone();
    if let Some(q) = &quoted {
        user_content.push_str(&format!(
            "\n\n(the previous message is responding to: {})",
            q.content
        ));
    }

    let mut messages: Vec<Value> = {
        let buf = CONTEXT_BUFFER.lock().unwrap();
        buf.iter()
            .map(|m| json!({"role": m.role, "content": m.content}))
            .collect()
    };
    messages.push(json!({"role":"user","content": user_content}));

    let system = format!("{}\n\n{}", get_personality(), LIMIT_CONTEXT);
    let reply = create_message(&system, messages).await?;

    add_to_context("user", &msg);
    add_to_context("assistant", &reply);

    println!("{reply}");
    Ok(reply)
}

/// `comeback(message)`.
pub async fn comeback(ctx: &Context, message: &Message) {
    if channel_blacklisted(ctx, message).await {
        react(ctx, message, "🙅‍♀️").await;
        return;
    }

    let quoted = match get_quoted_msg(ctx, message).await {
        Some(q) => q,
        None => {
            react(ctx, message, "🤷").await;
            return;
        }
    };

    let reply = match get_ai_comeback(&quoted.content).await {
        Ok(t) => t,
        Err(e) => {
            println!("error getting ai comeback");
            println!("{e}");
            get_comeback(&quoted.content)
        }
    };

    let _ = quoted.reply_ping(&ctx.http, reply).await;
}

/// `kindness(message)` — defined in the original but never invoked; preserved.
#[allow(dead_code)]
pub async fn kindness(ctx: &Context, message: &Message) {
    if channel_blacklisted(ctx, message).await {
        react(ctx, message, "🙅‍♀️").await;
        return;
    }

    let quoted = match get_quoted_msg(ctx, message).await {
        Some(q) => q,
        None => {
            react(ctx, message, "🤷").await;
            return;
        }
    };

    let reply = match get_ai_kindness(&quoted.content).await {
        Ok(t) => t,
        Err(e) => {
            println!("error getting ai kindness");
            println!("{e}");
            react(ctx, message, "❤️").await;
            return;
        }
    };

    let _ = quoted.reply_ping(&ctx.http, reply).await;
}

/// `respond_as(message, personality)`.
pub async fn respond_as(ctx: &Context, message: &Message, personality: &str) {
    if channel_blacklisted(ctx, message).await {
        react(ctx, message, "🙅‍♀️").await;
        return;
    }

    let quoted = match get_quoted_msg(ctx, message).await {
        Some(q) => q,
        None => {
            react(ctx, message, "🤷").await;
            return;
        }
    };

    // On AI error the original reacts 🫣 and then crashes on the undefined `msg`,
    // so no reply is ever sent — reproduced by reacting and returning.
    let reply = match get_person_response(personality, &quoted.content).await {
        Ok(t) => t,
        Err(e) => {
            println!("error getting ai comeback");
            println!("{e}");
            react(ctx, message, "🫣").await;
            return;
        }
    };

    let _ = quoted.reply_ping(&ctx.http, reply).await;
}

/// `recap(message)`.
pub async fn recap(ctx: &Context, message: &Message) {
    if channel_blacklisted(ctx, message).await {
        react(ctx, message, "🙅‍♀️").await;
        return;
    }

    let quoted = match get_quoted_msg(ctx, message).await {
        Some(q) => q,
        None => {
            react(ctx, message, "🤷").await;
            return;
        }
    };

    let target_user_id = quoted.author.id;
    let target_display = discord_util::display_name(&quoted);

    let typing = message.channel_id.start_typing(&ctx.http);

    let one_year_ago_ms =
        (chrono::Local::now() - chrono::Duration::days(365)).timestamp_millis();
    let mut collected: Vec<String> = Vec::new();

    if let Some(guild_id) = message.guild_id {
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            // `message.guild.text_channels` is sorted by (position, id); iterate in
            // the same deterministic order so the messages fed to the recap prompt
            // appear in the same order as in the original.
            let mut text_channels: Vec<_> = channels
                .into_iter()
                .filter(|(_, gc)| gc.kind == serenity::all::ChannelType::Text)
                .collect();
            text_channels.sort_by(|(aid, a), (bid, b)| {
                a.position.cmp(&b.position).then(aid.cmp(bid))
            });
            for (cid, _gc) in text_channels {
                let history =
                    discord_util::channel_history_after(ctx, cid, one_year_ago_ms).await;
                for m in history {
                    if m.author.id == target_user_id && !m.content.is_empty() {
                        collected.push(m.content);
                    }
                }
            }
        }
    }

    if collected.is_empty() {
        typing.stop();
        let _ = message
            .reply_ping(
                &ctx.http,
                format!("I couldn't find any messages from {target_display} in the past year."),
            )
            .await;
        return;
    }

    let mut full_text = collected.join("\n");
    if full_text.chars().count() > 100000 {
        full_text = full_text.chars().take(100000).collect();
    }

    let recap_response = match get_ai_recap(&target_display, &full_text).await {
        Ok(t) => t,
        Err(e) => {
            println!("Error generating recap: {e}");
            typing.stop();
            react(ctx, message, "😵").await;
            return;
        }
    };
    typing.stop();

    for chunk in auto_split_messages(&recap_response, 2000) {
        let _ = quoted.reply_ping(&ctx.http, chunk).await;
    }
}

/// `tldr(message)`.
pub async fn tldr(ctx: &Context, message: &Message) {
    if channel_blacklisted(ctx, message).await {
        react(ctx, message, "🙅‍♀️").await;
        return;
    }

    // Without a reply, the original dereferences `None.content`, then again in the
    // fallback, crashing the handler — observably nothing happens. Reproduced.
    let quoted = match get_quoted_msg(ctx, message).await {
        Some(q) => q,
        None => return,
    };

    let typing = message.channel_id.start_typing(&ctx.http);
    let reply = match get_tldr_response(&quoted.content).await {
        Ok(t) => t,
        Err(e) => {
            println!("error getting ai tldr response");
            println!("{e}");
            get_comeback(&quoted.content)
        }
    };
    typing.stop();

    let _ = quoted.reply_ping(&ctx.http, reply).await;
}

/// `bot_response(message)`.
pub async fn bot_response(ctx: &Context, message: &Message) {
    println!("responding to {}", message.content);
    if channel_blacklisted(ctx, message).await {
        react(ctx, message, "🙅‍♀️").await;
        return;
    }

    // The original wraps BOTH the AI call and the chunked replies in one try/except,
    // so a failure sending any reply also lands on the 🤷‍♀️ fallback.
    let typing = message.channel_id.start_typing(&ctx.http);
    let result = get_bot_response(ctx, message).await;
    typing.stop();

    let outcome: Result<(), String> = match result {
        Ok(msg) => {
            let mut res = Ok(());
            for chunk in auto_split_messages(&msg, 2000) {
                if let Err(e) = message.reply_ping(&ctx.http, chunk).await {
                    res = Err(e.to_string());
                    break;
                }
            }
            res
        }
        Err(e) => Err(e),
    };

    if let Err(e) = outcome {
        println!("error getting ai bot response");
        println!("{e}");
        react(ctx, message, "🤷‍♀️").await;
    }
}

/// `appropriate_reaction(message)` — the AI-chosen emoji reaction.
pub async fn appropriate_reaction(ctx: &Context, message: &Message) {
    let prompt = format!(
        "choose a single emoji as a reaction to the following message: {}",
        message.content
    );
    println!(
        "getting ai reaction for message: {}",
        message.content
    );
    let system = format!(
        "{}\n\nOnly respond with a single emoji character, nothing else.",
        get_personality()
    );
    match create_message(&system, vec![json!({"role":"user","content": prompt})]).await {
        Ok(text) => {
            let emoji = text.trim().to_string();
            println!("reacting with {emoji}");
            let _ = message
                .react(&ctx.http, discord_util::unicode(&emoji))
                .await;
        }
        Err(e) => {
            println!("error getting ai reaction");
            println!("{e}");
        }
    }
}
