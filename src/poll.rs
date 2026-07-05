//! Port of `poll.py` — the weekly games poll and the `hours_left` schedule math.
//! `poll()` is live (driven by the `trigger_poll` binary); the never-synced
//! `/game_poll` command also called it.

use chrono::Datelike;
use serenity::all::{
    Channel, ChannelId, Context, CreateMessage, CreatePoll, CreatePollAnswer, EmojiId,
    PollMediaEmoji,
};
use std::collections::HashMap;

#[allow(dead_code)]
pub const TEST_CHANNEL_ID: u64 = 1307019075700002913;
pub const SCHEDULE_CHANNEL_ID: u64 = 491257084650717195;

/// Only this many options are added to the poll.
pub const MAX_OPTIONS: usize = 10;

pub const OPTIONS: [(&str, &str); 48] = [
    ("Warhammer 40k Darktide", "🔨"),
    ("Mythforce", "🛡️"),
    ("Dune: Spice Wars", "🌵"),
    ("Deep Rock Galactic", "dwarf"),
    ("Jump Space", "🚀"),
    ("Gunfire Reborn", "🇨🇳"),
    ("Valheim", "⛺"),
    ("RV There Yet?", "🚬"),
    ("Splitgate", "splitgate"),
    ("Something else", "❓"),
    // library below 👇
    ("Helldivers 2", "helldivers"),
    ("Warhammer Vermintide 2", "🐀"),
    ("Marvel Rivals", "🦸"),
    ("Garfield Kart", "😾"),
    ("Age of Empires 2", "🏰"),
    ("Peak", "⛰️"),
    ("Halo 2", "halo2"),
    ("Abiotic Factor", "🧑‍🔬"),
    ("Phasmophobia", "👻"),
    ("Sea of Thieves", "sea_of_thieves"),
    ("Arc Raiders", "🌈"),
    ("Project Zomboid", "🧟"),
    ("Left 4 Dead 2", "🧟‍♀️"),
    ("Lethal Company", "🏢"),
    ("Mario Kart World", "mariokart"),
    ("Rematch", "⚽"),
    ("Fortnite", "fortnite"),
    ("Titanfall 2", "🤖"),
    ("Tiny Tina's Wonderlands", "🤪"),
    ("Rocket League", "rocket_league"),
    ("Risk of Rain 2", "🌧️"),
    ("Overwatch 2", "overwatch"),
    ("Void Crew", "🚀"),
    ("Killer Queen Black", "killerqueen"),
    ("MageQuit", "🧙"),
    ("LOTR: Return to Moria", "⛏️"),
    ("Counter Strike 2", "🔫"),
    ("Streets of Rogue", "🛣️"),
    ("Fall Guys", "fallguys"),
    ("Diablo 3", "😈"),
    ("Elite Dangerous", "elite"),
    ("Rounds", "🔴"),
    ("Core Keeper", "🔵"),
    ("Super Smash Bros Ultimate", "🥊"),
    ("MarioKart 8 Deluxe", "🏎️"),
    ("Starcraft 2", "starcraft"),
    ("Among Us", "amongus"),
    ("Helldivers 1", "🪂"),
];

/// `emoji.is_emoji(name)` proxy: in this fixed option set, every unicode emoji is
/// non-ASCII and every custom-emoji name is ASCII, so this discriminates them
/// exactly as the original did.
fn is_emoji(name: &str) -> bool {
    !name.is_ascii()
}

/// Resolve the schedule channel's guild emoji set, mirroring
/// `client.get_channel(id).guild.emojis`.
///
/// Returns `None` when the channel/guild can't be resolved — in the original this
/// is exactly when `get_emoji`'s `except` fires and returns 🙃 for a custom name.
/// `Some(map)` means the guild resolved, so a name that's simply absent yields no
/// emoji (matching `discord.utils.get` returning `None`, *not* raising).
async fn resolve_guild_emojis(
    ctx: &Context,
    channel_id: ChannelId,
) -> Option<HashMap<String, EmojiId>> {
    if let Ok(Channel::Guild(gc)) = channel_id.to_channel(ctx).await {
        if let Ok(list) = gc.guild_id.emojis(&ctx.http).await {
            return Some(list.into_iter().map(|e| (e.name.to_string(), e.id)).collect());
        }
    }
    None
}

/// Mirrors the inner `emoji(name)` helper:
///   * unicode emoji -> itself,
///   * custom name present in the guild -> that emoji,
///   * custom name absent but guild resolved -> `None` (no emoji, like
///     `discord.utils.get` returning `None`),
///   * guild unresolvable -> the 🙃 fallback (the `except` branch).
fn poll_emoji(map: &Option<HashMap<String, EmojiId>>, name: &str) -> Option<PollMediaEmoji> {
    if is_emoji(name) {
        Some(PollMediaEmoji::Name(name.to_string()))
    } else {
        match map {
            None => Some(PollMediaEmoji::Name("🙃".to_string())),
            Some(m) => m.get(name).map(|id| PollMediaEmoji::Id(*id)),
        }
    }
}

/// `poll(hours=50)`.
pub async fn poll(ctx: &Context, hours: i64) {
    println!("sending weekly games poll:  {hours}  hours");

    let channel_id = ChannelId::new(SCHEDULE_CHANNEL_ID);
    let emoji_map = resolve_guild_emojis(ctx, channel_id).await;

    let mut answers: Vec<CreatePollAnswer> = Vec::new();
    for (text, icon) in OPTIONS.iter().take(MAX_OPTIONS) {
        let mut answer = CreatePollAnswer::new().text(*text);
        // When the emoji resolves to None (custom name absent from a resolved
        // guild), add the answer with no emoji — exactly like `emoji=None`.
        if let Some(pe) = poll_emoji(&emoji_map, icon) {
            answer = answer.emoji(pe);
        }
        answers.push(answer);
    }

    let duration = std::time::Duration::from_secs((hours.max(0) as u64) * 3600);

    let game_poll = CreatePoll::new()
        .question("🎮 What to play on Tuesday? 🎮")
        .answers(answers)
        .duration(duration)
        .allow_multiselect();

    let _ = channel_id
        .send_message(&ctx.http, CreateMessage::new().poll(game_poll))
        .await;
}

fn python_mod(a: i64, b: i64) -> i64 {
    ((a % b) + b) % b
}

/// `hours_left()` — hours until the next Tuesday 5pm party, plus a hardcoded
/// 7-hour "utc_offset" fudge, floored.
pub fn hours_left() -> i64 {
    // datetime.now() is naive local time; operate entirely in naive local time.
    let now = chrono::Local::now().naive_local();

    let day: i64 = 1; // Tuesday (Python weekday: Monday=0)
    let hour: u32 = 17; // 5 PM

    let weekday = now.weekday().num_days_from_monday() as i64; // Monday=0
    let days_until_party = python_mod(day - weekday, 7);

    let target_date = now.date() + chrono::Duration::days(days_until_party);
    // `.replace(hour=17, minute=0, second=0, microsecond=0)`
    let mut next_party = target_date.and_hms_opt(hour, 0, 0).unwrap();

    if next_party <= now {
        next_party += chrono::Duration::weeks(1);
    }

    let time_difference = next_party - now;
    // Match Python's `total_seconds()` microsecond precision before flooring.
    let hours_remaining = time_difference
        .num_microseconds()
        .map(|us| us as f64 / 1_000_000.0)
        .unwrap_or_else(|| time_difference.num_seconds() as f64)
        / 3600.0;

    let utc_offset = 7i64;
    hours_remaining.floor() as i64 + utc_offset
}
