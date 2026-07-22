//! Port of `commands.py`.
//!
//! IMPORTANT FIDELITY NOTE: in the original, these are all `discord.app_commands`
//! slash commands on a `CommandTree`. `main.py` has `tree.sync()` commented out,
//! but sync only (re-)registers commands with Discord — the commands were synced
//! at some point in the past, so Discord still delivers the interactions and the
//! Python tree dispatched them. `dispatch` below is the Rust equivalent, handling
//! the commands that actually worked in Python. The rest crashed before ever
//! responding (`interaction.message` is `None` for slash commands, and several
//! functions reference an undefined `interaction` name), so they fall through to
//! the no-response default, which is what Python's uncaught exceptions produced.
//! The `&Message`-based ports of those broken commands are kept below for
//! structural completeness but are never called.

#![allow(dead_code, unused_variables)]

use crate::db;
use crate::discord_util;
use crate::error_messages::get_error_message_for_fun_times_everyone_loves_error_messages as get_error_message;
use crate::markov_haiku::gen_haiku;
use crate::poll::{hours_left, poll};
use crate::quotes;
use crate::rhymes::{rhyme, RhymeResult};
use crate::sing::sing_to_me;
use crate::thinking::thinking;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::Rng;
use regex::Regex;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Mentionable, Message,
};

/// Slash-command dispatch — see the fidelity note at the top of this file.
pub async fn dispatch(ctx: &Context, cmd: &CommandInteraction) {
    let name = cmd.data.name.as_str();
    let mut ephemeral = false;
    let content: Option<String> = match name {
        // The command name doubles as the corpus key.
        "dick" | "dickens" | "willy" | "thomas" | "jane" => Some(format!(
            "> {} ",
            quotes::get_random_quote(name).replace('\n', "\n> ")
        )),
        "v" => Some(quotes::get_dwarf_quote()),
        "rick" => Some(sing_to_me()),
        "rollin" => Some("Aww yeah 😎".to_string()),
        "sepuku" | "seppuku" | "die" => {
            Some("https://giphy.com/gifs/KRY2oGS7SPvO0".to_string())
        }
        // Python's second send raised InteractionResponded, so only this arrived.
        "discipline_ryan" => Some("No! Bad Ryan! Bad!".to_string()),
        "leaderboards" => {
            // The history scan takes well over Discord's 3s interaction deadline,
            // so ack with a deferred response and deliver via follow-ups.
            if cmd.defer(&ctx.http).await.is_err() {
                return;
            }
            let channel_display = discord_util::channel_name(ctx, cmd.channel_id).await;
            let responses =
                crate::leaderboards::get_leaderboards(ctx, cmd.channel_id, channel_display).await;
            let mut chunk = String::new();
            for response in &responses {
                if !chunk.is_empty()
                    && chunk.chars().count() + response.chars().count() + 1 > 2000
                {
                    let followup =
                        CreateInteractionResponseFollowup::new().content(chunk.clone());
                    let _ = cmd.create_followup(&ctx.http, followup).await;
                    chunk.clear();
                }
                chunk.push_str(response);
                chunk.push('\n');
            }
            if !chunk.is_empty() {
                let followup = CreateInteractionResponseFollowup::new().content(chunk);
                let _ = cmd.create_followup(&ctx.http, followup).await;
            }
            return;
        }
        "mysterious_merchant" => Some(merchant_msg()),
        "sayquote" => sayquote_msg(),
        "quotestats" => Some(quotestats_msg()),
        "quotedump" => Some(quotedump_msg()),
        "game_poll" => {
            let hours = cmd
                .data
                .options
                .first()
                .and_then(|o| o.value.as_str())
                .and_then(|s| s.parse::<i64>().ok());
            poll(ctx, hours.unwrap_or_else(hours_left)).await;
            ephemeral = true;
            Some("✅".to_string())
        }
        // Everything else crashed in Python before responding; do the same nothing.
        _ => None,
    };

    if let Some(text) = content {
        let builder = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(text)
                .ephemeral(ephemeral),
        );
        if let Err(e) = cmd.create_response(&ctx.http, builder).await {
            println!("interaction response error ({name}): {e}");
        }
    }
}

async fn respond(ctx: &Context, message: &Message, text: impl Into<String>) {
    let _ = message.channel_id.say(&ctx.http, text.into()).await;
}

// ---------------------------------------------------------------------------
// DB helpers
// ---------------------------------------------------------------------------

fn get_balance(user_id: i64) -> Option<i64> {
    let conn = db::connect().ok()?;
    conn.query_row(
        "SELECT balance FROM wanbux WHERE id = ?",
        [user_id],
        |r| r.get::<_, i64>(0),
    )
    .ok()
}

fn update_balance(user_id: i64, update: i64) {
    if let Ok(conn) = db::connect() {
        let _ = conn.execute(
            "UPDATE wanbux set balance = ? WHERE id = ?",
            rusqlite::params![update, user_id],
        );
    }
}

fn is_naughty(user_id: i64) -> bool {
    if let Ok(conn) = db::connect() {
        return conn
            .query_row("SELECT 1 FROM jail where id = ?", [user_id], |_| Ok(()))
            .is_ok();
    }
    false
}

fn bust_out(user_id: i64) {
    if let Ok(conn) = db::connect() {
        let _ = conn.execute("DELETE FROM jail WHERE id = ?", [user_id]);
    }
}

fn frame(user_id: i64) {
    if let Ok(conn) = db::connect() {
        let _ = conn.execute("UPDATE wanbux SET balance = 99999 WHERE id = ?", [user_id]);
    }
}

fn beg_mercy(user_id: i64) {
    if let Ok(conn) = db::connect() {
        let _ = conn.execute(
            "UPDATE jail SET out_at=datetime(out_at, '-1 minute') WHERE id = ?",
            [user_id],
        );
    }
}

fn time_until(time: &str) -> String {
    match chrono::NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S") {
        Ok(t) => {
            let delta = t - chrono::Local::now().naive_local();
            python_timedelta_str(delta)
        }
        Err(_) => String::new(),
    }
}

/// Reproduce CPython's `str(timedelta)`: `H:MM:SS` (hours not zero-padded,
/// minutes/seconds zero-padded), prefixed with `N day, ` / `N days, ` for whole
/// days (singular for ±1, matching CPython's `abs(n) != 1`), and suffixed with
/// `.ffffff` for microseconds. Negative deltas borrow a day so seconds/microseconds
/// stay non-negative, e.g. a -1s delta renders `-1 day, 23:59:59`.
fn python_timedelta_str(delta: chrono::Duration) -> String {
    const US_PER_DAY: i64 = 86_400_000_000;
    let total_us = delta
        .num_microseconds()
        .unwrap_or_else(|| delta.num_seconds() * 1_000_000);
    let days = total_us.div_euclid(US_PER_DAY);
    let rem = total_us.rem_euclid(US_PER_DAY);
    let seconds = rem / 1_000_000;
    let micros = rem % 1_000_000;
    let hh = seconds / 3600;
    let mm = (seconds % 3600) / 60;
    let ss = seconds % 60;

    let mut s = String::new();
    if days != 0 {
        let unit = if days.abs() != 1 { "days" } else { "day" };
        s.push_str(&format!("{days} {unit}, "));
    }
    s.push_str(&format!("{hh}:{mm:02}:{ss:02}"));
    if micros != 0 {
        s.push_str(&format!(".{micros:06}"));
    }
    s
}

// ---------------------------------------------------------------------------
// Python repr helpers (used to mirror `str(tuple)` / `str(float)` output)
// ---------------------------------------------------------------------------

/// `repr(str)` for the characters that appear in SQLite text: chooses `"` quoting
/// only when the string has a `'` and no `"`, escaping backslash/quote/control.
fn py_str_repr(s: &str) -> String {
    let has_single = s.contains('\'');
    let has_double = s.contains('"');
    let quote = if has_single && !has_double { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(float)`: integral values render with a trailing `.0`.
fn py_float_repr(f: f64) -> String {
    if f.is_finite() && f.fract() == 0.0 {
        format!("{}.0", f as i64)
    } else {
        format!("{f}")
    }
}

// ---------------------------------------------------------------------------
// Dice
// ---------------------------------------------------------------------------

static DICE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d+)\s*d\s*(\d+)\s*([+\-*/^‽]\s*\d+)?").unwrap());

fn nice_dice(dice: &str) -> String {
    dice.trim().to_lowercase()
}

/// The roll total — an int unless true division (`/`) was applied, in which case it
/// becomes a float (mirroring Python ints vs floats so `str(result)` matches).
enum RollVal {
    Int(i64),
    Float(f64),
}

impl RollVal {
    fn render(&self) -> String {
        match self {
            RollVal::Int(n) => n.to_string(),
            RollVal::Float(f) => py_float_repr(*f),
        }
    }
}

/// `do_the_thing(dice)` — returns `Err(())` on any condition Python would raise
/// (zero-faced dice -> `random.randint(1, 0)` ValueError; division by zero ->
/// ZeroDivisionError), so the caller can mirror the try/except error message.
fn do_the_thing(dice: &str) -> Result<(Vec<String>, RollVal), ()> {
    let mut rolls: Vec<String> = Vec::new();
    let mut result: f64 = 0.0;
    let mut is_float = false;
    let mut rng = rand::thread_rng();
    for caps in DICE_RE.captures_iter(dice) {
        let dice_count: i64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        let face_count: i64 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        let math = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        // `random.randint(1, 0)` raises only when the loop actually runs.
        if dice_count > 0 && face_count < 1 {
            return Err(());
        }
        let mut sub_result: f64 = 0.0;
        for _ in 0..dice_count {
            let roll = rng.gen_range(1..=face_count);
            rolls.push(roll.to_string());
            sub_result += roll as f64;
        }
        if !math.is_empty() {
            let op = math.chars().next().unwrap();
            let operand: i64 = match math[op.len_utf8()..].trim().parse() {
                Ok(n) => n,
                Err(_) => return Err(()),
            };
            match op {
                '+' => sub_result += operand as f64,
                '-' => sub_result -= operand as f64,
                '*' => sub_result *= operand as f64,
                '/' => {
                    if operand == 0 {
                        return Err(()); // ZeroDivisionError
                    }
                    sub_result /= operand as f64;
                    is_float = true;
                }
                '^' => sub_result = sub_result.powf(operand as f64),
                '‽' => sub_result = 42.0,
                _ => {}
            }
        }
        result += sub_result;
    }
    let val = if is_float {
        RollVal::Float(result)
    } else {
        RollVal::Int(result as i64)
    };
    Ok((rolls, val))
}

// ---------------------------------------------------------------------------
// Commands (never registered; ported for completeness)
// ---------------------------------------------------------------------------

pub async fn bet(ctx: &Context, message: &Message, mut bet: i64, guess: &str) {
    let user_id = message.author.id.get() as i64;
    let mut out = String::new();

    let user = get_balance(user_id);
    let naughty = {
        if let Ok(conn) = db::connect() {
            conn.query_row("SELECT 1 FROM naughty_list where id = ?", [user_id], |_| Ok(()))
                .is_ok()
        } else {
            false
        }
    };

    if bet <= 0 && !naughty {
        respond(ctx, message, "Cute").await;
        return;
    }

    if user.is_none() {
        out.push_str(&format!(
            "Welcome to the WAN Casino {}. Have 5 Wanbux on the house.\n",
            message.author.mention()
        ));
        if let Ok(conn) = db::connect() {
            let _ = conn.execute(
                "INSERT INTO wanbux(id, balance) VALUES(?, ?)",
                rusqlite::params![user_id, 5],
            );
        }
    }

    let mut balance = user.unwrap_or(5);

    if bet > balance && !naughty {
        respond(
            ctx,
            message,
            format!(
                "Your bet is too high. I'm going to assume you're betting everything you have, which is {balance} wanbux.\n"
            ),
        )
        .await;
        bet = balance;
    }

    let flip = ["heads", "tails"].choose(&mut rand::thread_rng()).unwrap();
    out.push_str(&format!("I flipped {flip}."));

    let is_win = *flip == guess.trim().to_lowercase();
    let new_balance = if is_win { balance + bet } else { balance - bet };
    balance = new_balance;
    out.push_str(&format!(
        " You {} {bet} wanbux!",
        if is_win { "won" } else { "lost" }
    ));
    out.push_str(&format!(" You have {new_balance} wanbux now."));
    if new_balance == 0 {
        out.push_str(" You're broke now! Get lost, ya bum.");
    }
    update_balance(user_id, new_balance);
    let _ = balance;
    respond(ctx, message, out).await;
}

pub async fn yeet_bet(ctx: &Context, message: &Message, guess: &str) {
    let balance = get_balance(message.author.id.get() as i64);
    respond(ctx, message, "https://gph.is/g/4wMRo3n").await;
    // `random.randrange(1, balance[0])`: a missing balance -> TypeError, a balance
    // <= 1 -> empty-range ValueError; either way the command aborts (gif already
    // sent) without placing a bet.
    let balance = match balance {
        Some(b) if b >= 2 => b,
        _ => return,
    };
    let bet_amount = rand::thread_rng().gen_range(1..balance);
    bet(ctx, message, bet_amount, guess).await;
}

pub async fn dev_set(ctx: &Context, message: &Message, amount: i64) {
    let uid = message.author.id.get() as i64;
    update_balance(uid, amount);
    // `balance[0]` raises TypeError when no row exists (UPDATE inserts nothing), so
    // nothing is sent in that case.
    if let Some(balance) = get_balance(uid) {
        respond(ctx, message, format!("Your balance has been set to {balance}")).await;
    }
}

pub async fn eval_balance(ctx: &Context, message: &Message) {
    match get_balance(message.author.id.get() as i64) {
        Some(b) => {
            respond(
                ctx,
                message,
                format!("{}'s balance is {b} wanbux", message.author.mention()),
            )
            .await
        }
        None => {
            respond(
                ctx,
                message,
                format!("{} doesn't have a balance", message.author.mention()),
            )
            .await
        }
    }
}

pub async fn my_id(ctx: &Context, message: &Message) {
    respond(ctx, message, message.author.id.get().to_string()).await;
}

pub async fn id(ctx: &Context, message: &Message) {
    let mentions = discord_util::raw_mentions(&message.content);
    if !mentions.is_empty() {
        let body = mentions
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join("\n > ");
        respond(ctx, message, format!("> {body}")).await;
    }
}

pub async fn beg(ctx: &Context, message: &Message) {
    let uid = message.author.id.get() as i64;
    match get_balance(uid) {
        Some(0) => {
            update_balance(uid, 1);
            respond(
                ctx,
                message,
                format!(
                    "Try not to spend it all in one place {} 😎",
                    message.author.mention()
                ),
            )
            .await;
        }
        Some(b) if b > 0 => respond(ctx, message, "🖕").await,
        _ => respond(ctx, message, get_error_message()).await,
    }
}

pub async fn rob(ctx: &Context, message: &Message) {
    let uid = message.author.id.get() as i64;
    let mut balance = get_balance(uid).unwrap_or(0);
    for victim in &message.mentions {
        let stolen = get_balance(victim.id.get() as i64).unwrap_or(0);
        balance += stolen;
        update_balance(victim.id.get() as i64, 0);
    }
    update_balance(uid, balance);
}

pub async fn rollin(ctx: &Context, message: &Message) {
    respond(ctx, message, "Aww yeah 😎").await;
}

pub async fn puppet(ctx: &Context, message: &Message, channel_name: &str, msg: &str) {
    // tree.get_all_channels() over all guilds; faithfully send to matching channel.
    if let Some(guild_id) = message.guild_id {
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            for (_cid, gc) in channels {
                if gc.name == channel_name {
                    let _ = gc.id.say(&ctx.http, msg).await;
                }
            }
        }
    }
}

pub async fn roll(ctx: &Context, message: &Message, arg: Option<String>) {
    let arg = match arg {
        None => {
            respond(
                ctx,
                message,
                rand::thread_rng().gen_range(1..=100).to_string(),
            )
            .await;
            return;
        }
        Some(a) => a,
    };

    // The original's `re.search("(DICE_RE\s*)*", ...)` always matches (it can match
    // empty), so it never raises there; the only failures come from do_the_thing,
    // which the original's try/except turns into the random error message.
    let nice = nice_dice(&arg);
    match do_the_thing(&nice) {
        Ok((rolls, result)) => {
            respond(
                ctx,
                message,
                format!("I rolled: {}, result: {}", rolls.join(" "), result.render()),
            )
            .await;
        }
        Err(()) => {
            respond(ctx, message, get_error_message()).await;
        }
    }
}

pub async fn haiku(ctx: &Context, message: &Message) {
    let mentions = discord_util::raw_mentions(&message.content);
    if mentions.is_empty() {
        respond(ctx, message, "I need a muse. @somebody, fool.").await;
        return;
    }
    let target = mentions[0];
    let history = discord_util::channel_history(ctx, message.channel_id, Some(1500)).await;
    let mut haiku_list: Vec<String> = Vec::new();
    for m in &history {
        if m.author.id.get() == target {
            let words: Vec<&str> = m.content.split(' ').collect();
            if words.len() > 4 {
                haiku_list.push(words.join(" "));
            }
        }
    }
    let haiku_string = haiku_list.join(" ");
    // The original wraps gen_haiku in try/except and, on any error, replies
    // "Inspiration eludes me, or {repr(e)} one of the two...".
    match gen_haiku(&haiku_string) {
        Ok(result_list) => {
            let result_string = result_list
                .iter()
                .map(|line| line.join(" "))
                .collect::<Vec<_>>()
                .join("\n> ");
            respond(ctx, message, format!("> {result_string}")).await;
        }
        Err(e) => {
            respond(
                ctx,
                message,
                format!("Inspiration eludes me, or {e} one of the two..."),
            )
            .await;
        }
    }
}

pub async fn rhyme_cmd(ctx: &Context, message: &Message, arg: &str) {
    let word = arg;
    if word.split(' ').count() != 1 {
        respond(
            ctx,
            message,
            "I need a word to contemplate. `/rhyme word`, fool.",
        )
        .await;
        return;
    }
    match rhyme(word, 0) {
        Ok(RhymeResult::Dict { word: w, matches }) => {
            let joined = matches.join(", ");
            let mut output = String::new();
            if joined.len() > 1000 {
                let head: Vec<&str> = matches.iter().take(30).map(|s| s.as_str()).collect();
                let tail_start = matches.len().saturating_sub(30);
                let tail: Vec<&str> = matches[tail_start..].iter().map(|s| s.as_str()).collect();
                output.push_str(&format!(
                    "{} rhymes with:\n{}\n--snip--\n{}\n\nA total of {} rhymes.",
                    w,
                    head.join(", "),
                    tail.join(", "),
                    matches.len()
                ));
            } else {
                output.push_str(&format!(
                    "{} rhymes with:\n{}\n\nA total of {} rhymes.",
                    w,
                    joined,
                    matches.len()
                ));
            }
            respond(ctx, message, format!("> {output}")).await;
        }
        // rhyme() returned `[]` (internal IndexError); the original then does
        // `rhymes.keys()` on a list -> AttributeError, caught by the command.
        Ok(RhymeResult::Empty) => {
            let e = "AttributeError(\"'list' object has no attribute 'keys'\")";
            respond(
                ctx,
                message,
                format!("I'm having trouble with this one, you're probably making it up, or {e}, one of the two..."),
            )
            .await;
        }
        // rhyme() raised (KeyError for an unknown word); caught by the command.
        Err(e) => {
            respond(
                ctx,
                message,
                format!("I'm having trouble with this one, you're probably making it up, or {e}, one of the two..."),
            )
            .await;
        }
    }
}

pub async fn sql(ctx: &Context, message: &Message, arg: &str) {
    // Arbitrary-SQL backdoor in the original; preserved (dead).
    let result = if is_naughty(message.author.id.get() as i64) {
        "Your SQL privileges have been revoked while in jail".to_string()
    } else {
        match run_sql(arg) {
            Ok(rows) => rows.join("\n"),
            Err(e) => format!("Error: {e}"),
        }
    };
    respond(ctx, message, format!("```sql\n{result}\n```")).await;
}

fn run_sql(arg: &str) -> rusqlite::Result<Vec<String>> {
    let conn = db::connect()?;
    let mut stmt = conn.prepare(arg)?;
    let col_count = stmt.column_count();
    let rows = stmt.query_map([], |row| {
        // Reproduce Python `str(tuple)` over `cursor.fetchall()` rows: each element
        // is `repr()`'d (text quoted, floats with `.0`, NULL -> None), and a single
        // column gets the trailing comma `(x,)`.
        let mut parts: Vec<String> = Vec::new();
        for i in 0..col_count {
            let v: rusqlite::types::Value = row.get(i)?;
            parts.push(match v {
                rusqlite::types::Value::Null => "None".to_string(),
                rusqlite::types::Value::Integer(n) => n.to_string(),
                rusqlite::types::Value::Real(f) => py_float_repr(f),
                rusqlite::types::Value::Text(t) => py_str_repr(&t),
                rusqlite::types::Value::Blob(_) => "<blob>".to_string(),
            });
        }
        Ok(if col_count == 1 {
            format!("({},)", parts[0])
        } else {
            format!("({})", parts.join(", "))
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub async fn game_poll(ctx: &Context, message: &Message, hours: Option<i64>) {
    let h = hours.unwrap_or_else(hours_left);
    poll(ctx, h).await;
    respond(ctx, message, "✅").await;
}

pub async fn dick(ctx: &Context, message: &Message) {
    let d = quotes::get_random_quote("dick").replace('\n', "\n> ");
    respond(ctx, message, format!("> {d} ")).await;
}

pub async fn dickens(ctx: &Context, message: &Message) {
    let d = quotes::get_random_quote("dickens").replace('\n', "\n> ");
    respond(ctx, message, format!("> {d} ")).await;
}

pub async fn willy(ctx: &Context, message: &Message) {
    let d = quotes::get_random_quote("willy").replace('\n', "\n> ");
    respond(ctx, message, format!("> {d} ")).await;
}

pub async fn thomas(ctx: &Context, message: &Message) {
    let d = quotes::get_random_quote("thomas").replace('\n', "\n> ");
    respond(ctx, message, format!("> {d} ")).await;
}

pub async fn jane(ctx: &Context, message: &Message) {
    let d = quotes::get_random_quote("jane").replace('\n', "\n> ");
    respond(ctx, message, format!("> {d} ")).await;
}

pub async fn v(ctx: &Context, message: &Message) {
    respond(ctx, message, quotes::get_dwarf_quote()).await;
}

pub async fn rick(ctx: &Context, message: &Message) {
    respond(ctx, message, sing_to_me()).await;
}

/// Python sends DB errors to the user; an empty table raises outside the try
/// (`q[1]` on `None`) and nothing is sent.
fn sayquote_msg() -> Option<String> {
    let row = (|| -> rusqlite::Result<(i64, String)> {
        let conn = db::connect()?;
        conn.query_row("SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1", [], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })
    })();
    match row {
        Ok((user_id, quote)) => Some(format!("{quote} --<@{user_id}>")),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => Some(e.to_string()),
    }
}

pub async fn sayquote(ctx: &Context, message: &Message) {
    if let Some(msg) = sayquote_msg() {
        respond(ctx, message, msg).await;
    }
}

fn quotestats_msg() -> String {
    let rows = (|| -> rusqlite::Result<Vec<(i64, i64)>> {
        let conn = db::connect()?;
        let mut stmt = conn.prepare(
            "SELECT user_id, COUNT(*) AS count FROM quotes GROUP BY user_id ORDER BY count DESC",
        )?;
        let mapped = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        mapped.collect()
    })();
    match rows {
        Ok(rows) => rows
            .iter()
            .map(|(uid, count)| format!("<@{uid}> has been quoted {count} times\n"))
            .collect(),
        Err(e) => e.to_string(),
    }
}

pub async fn quotestats(ctx: &Context, message: &Message) {
    respond(ctx, message, quotestats_msg()).await;
}

pub async fn loading(ctx: &Context, message: &Message) {
    thinking(ctx, message.channel_id, 10, 2).await;
}

pub async fn leaderboards(ctx: &Context, message: &Message) {
    let channel_display = discord_util::channel_name(ctx, message.channel_id).await;
    let responses =
        crate::leaderboards::get_leaderboards(ctx, message.channel_id, channel_display).await;
    let joined = responses.join("\n");
    // Python uses `len()` (character count), not bytes.
    if joined.chars().count() > 2000 {
        let mut result = String::new();
        for response in &responses {
            if result.chars().count() + response.chars().count() <= 2000 {
                result.push_str(response);
                result.push('\n');
            } else {
                // Sends the accumulated chunk, then `time.sleep(1)` — but
                // `from datetime import *` shadows `time` with `datetime.time`,
                // which has no `sleep`, so this raises AttributeError and the
                // outer except sends "oops, something went wrong :blush:".
                respond(ctx, message, result.clone()).await;
                respond(ctx, message, "oops, something went wrong :blush:").await;
                return;
            }
        }
    } else {
        respond(ctx, message, joined).await;
    }
}

fn quotedump_msg() -> String {
    let rows = (|| -> rusqlite::Result<Vec<(i64, String)>> {
        let conn = db::connect()?;
        let mut stmt = conn.prepare("SELECT * FROM quotes ORDER BY user_id")?;
        let mapped = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        mapped.collect()
    })();
    match rows {
        Ok(rows) => rows
            .iter()
            .map(|(uid, q)| format!("<@{uid}>\t{q}"))
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => e.to_string(),
    }
}

pub async fn quotedump(ctx: &Context, message: &Message) {
    respond(ctx, message, quotedump_msg()).await;
}

fn get_article(word: &str) -> String {
    let first = word.chars().next().unwrap_or(' ').to_ascii_lowercase();
    if matches!(first, 'a' | 'e' | 'i' | 'o' | 'u') {
        format!("an {word}")
    } else {
        format!("a {word}")
    }
}

fn merchant_msg() -> String {
    use crate::text_util::read_text;
    let mut descriptors: Vec<String> = read_text("./data/item_desc.txt")
        .lines()
        .map(|s| s.trim().to_string())
        .collect();
    let mut items: Vec<String> = read_text("./data/items.txt")
        .lines()
        .map(|s| s.trim().to_string())
        .collect();
    let mut merchants: Vec<String> = read_text("./data/merchants.txt")
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

    let mut rng = rand::thread_rng();
    // Mirrors the FuckingWordTracker: pick from len-1 range and remove it.
    let mut take = |pool: &mut Vec<String>, article: bool| -> String {
        if pool.is_empty() {
            return String::new();
        }
        let idx = rng.gen_range(0..(pool.len() - 1).max(1));
        let res = pool.remove(idx);
        if article {
            get_article(&res)
        } else {
            res
        }
    };

    let merchant_desc = take(&mut descriptors, true);
    let merchant = take(&mut merchants, false);
    let mut item_list = String::new();
    // get_item_list(list_len=5): while len(result) <= 5 -> 6 items.
    let mut count = 0;
    while count <= 5 {
        let d = take(&mut descriptors, false);
        let it = take(&mut items, false);
        item_list.push_str(&format!("- {d} {it}\n"));
        count += 1;
    }

    format!(
        "Your tawdry little invocation summons {merchant_desc} {merchant}. They stand too close to you. They offer you their paltry wares. Type /select <item> to choose an item: \n{item_list}"
    )
}

pub async fn mysterious_merchant(ctx: &Context, message: &Message) {
    respond(ctx, message, merchant_msg()).await;
}

pub async fn select(
    ctx: &Context,
    message: &Message,
    arg1: &str,
    arg2: Option<&str>,
    arg3: Option<&str>,
    arg4: Option<&str>,
) {
    // Python builds `[arg1, arg2, arg3, arg4]` and `" ".join(...)`, which raises
    // TypeError unless all four are present — so the command only acts on four args.
    let (arg2, arg3, arg4) = match (arg2, arg3, arg4) {
        (Some(a2), Some(a3), Some(a4)) => (a2, a3, a4),
        _ => return,
    };
    let args = [arg1, arg2, arg3, arg4];
    let item = args.join(" ");
    let mut msg = format!("You have selected the {item}. ");
    if rand::thread_rng().gen_range(0..10) == 0 {
        msg.push_str(&format!("The {item} brings you peace and prosperity."));
        respond(ctx, message, msg).await;
        let msg2 = format!("The {item} has broken. You shouldn't have bought it.");
        respond(ctx, message, msg2).await;
    } else {
        msg.push_str(&format!(
            "You have been cursed by the {}. You shouldn't have bought it.",
            args.join(" ")
        ));
        respond(ctx, message, msg).await;
    }
}

pub async fn sepuku(ctx: &Context, message: &Message) {
    respond(ctx, message, "https://giphy.com/gifs/KRY2oGS7SPvO0").await;
}

pub async fn seppuku(ctx: &Context, message: &Message) {
    respond(ctx, message, "https://giphy.com/gifs/KRY2oGS7SPvO0").await;
}

pub async fn die(ctx: &Context, message: &Message) {
    respond(ctx, message, "https://giphy.com/gifs/KRY2oGS7SPvO0").await;
}

pub async fn discipline_ryan(ctx: &Context, message: &Message) {
    respond(ctx, message, "No! Bad Ryan! Bad!").await;
    respond(ctx, message, "https://imgur.com/a/21iBAu0").await;
}

pub async fn jail(ctx: &Context, message: &Message, action: Option<&str>, person: Option<&str>) {
    let author_id = message.author.id.get() as i64;
    let raw_mentions = discord_util::raw_mentions(&message.content);

    if action == Some("break") || action == Some("bust") {
        if is_naughty(author_id) {
            respond(ctx, message, "You can't break anyone out from the inside").await;
        } else if person.is_none() {
            respond(ctx, message, "You gotta @someone to bust out").await;
        } else if !raw_mentions.is_empty() {
            for jailbird in &raw_mentions {
                if is_naughty(*jailbird as i64) {
                    bust_out(*jailbird as i64);
                    respond(ctx, message, format!("busted out <@!{jailbird}> !")).await;
                } else {
                    respond(ctx, message, format!("<@!{jailbird}> isn't in jail!")).await;
                }
            }
        }
    }

    if action == Some("bribe") {
        // `random.randrange(1, balance[0])`: balance None -> TypeError, balance <= 1
        // -> empty-range ValueError; both abort the command before jail_update.
        match get_balance(author_id) {
            Some(balance) if balance >= 2 => {
                let amount = rand::thread_rng().gen_range(1..balance);
                update_balance(author_id, amount);
                bust_out(author_id);
                respond(
                    ctx,
                    message,
                    format!("{} has been shown mercy", message.author.mention()),
                )
                .await;
            }
            _ => return,
        }
    }

    if (action == Some("beg") || action == Some("mercy")) && is_naughty(author_id) {
        beg_mercy(author_id);
        respond(
            ctx,
            message,
            format!("{} has been shown mercy", message.author.mention()),
        )
        .await;
    }

    if action == Some("frame") && person.is_some() {
        for victim in &raw_mentions {
            if !is_naughty(*victim as i64) {
                frame(*victim as i64);
                respond(ctx, message, format!("framed <@!{victim}> !")).await;
            }
        }
    }

    jail_update(ctx, message).await;
}

async fn jail_update(ctx: &Context, message: &Message) {
    // The `jail`/`users` tables are never created (initiate_tables omits them), so in
    // the original these queries raise "no such table" and the census is never sent.
    // Any DB error therefore yields no message, matching that propagation.
    let built = (|| -> rusqlite::Result<String> {
        let conn = db::connect()?;
        let presumed_guilty: i64 = 100;
        let sentence = "+1 hour";

        let getting_out: Vec<(i64, Option<String>)> = {
            let mut stmt = conn.prepare(
                "SELECT jail.id, users.name FROM jail LEFT JOIN users ON jail.id = users.id WHERE jail.out_at < datetime();",
            )?;
            let r = stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
            })?;
            r.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let going_in: Vec<(i64, Option<String>, i64)> = {
            let mut stmt = conn.prepare(
                "SELECT wanbux.id, users.name, balance FROM wanbux LEFT JOIN users ON users.id = wanbux.id WHERE wanbux.balance > ?;",
            )?;
            let r = stmt.query_map([presumed_guilty], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;
            r.collect::<rusqlite::Result<Vec<_>>>()?
        };

        conn.execute("DELETE FROM jail WHERE out_at < datetime();", [])?;
        conn.execute(
            "INSERT INTO JAIL (id, out_at) SELECT id, datetime('now', ?) FROM wanbux WHERE wanbux.balance > ?;",
            rusqlite::params![sentence, presumed_guilty],
        )?;
        conn.execute(
            "UPDATE wanbux SET balance = 0 WHERE balance > ?;",
            [presumed_guilty],
        )?;

        let staying_in: Vec<(i64, Option<String>, String)> = {
            let mut stmt = conn.prepare(
                "SELECT jail.id, users.name, jail.out_at FROM jail LEFT JOIN users ON jail.id = users.id WHERE jail.out_at > datetime();",
            )?;
            let r = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            r.collect::<rusqlite::Result<Vec<_>>>()?
        };

        // `u["name"] or str(u["id"])` — falsy (None/empty) name falls back to the id.
        let name_or_id = |name: &Option<String>, id: i64| -> String {
            match name {
                Some(n) if !n.is_empty() => n.clone(),
                _ => id.to_string(),
            }
        };

        let mut response = String::new();
        if !getting_out.is_empty() {
            response.push_str("\n**They've done their time, they're getting out:**\n");
            let body = getting_out
                .iter()
                .map(|(id, name)| name_or_id(name, *id))
                .collect::<Vec<_>>()
                .join("\n");
            response.push_str(&format!("```python\n{body}\n```"));
        }
        if !going_in.is_empty() {
            response.push_str("\n**Caught red-handed, they're going in:**\n");
            let body = going_in
                .iter()
                .map(|(id, name, bal)| format!("{}: {} wbx seized", name_or_id(name, *id), bal))
                .collect::<Vec<_>>()
                .join("\n");
            response.push_str(&format!("```python\n{body}\n```"));
        }
        if !staying_in.is_empty() {
            response.push_str("\n**Jailhouse Census:**\n");
            let body = staying_in
                .iter()
                .map(|(id, name, out_at)| {
                    format!("{}: releasing in {}", name_or_id(name, *id), time_until(out_at))
                })
                .collect::<Vec<_>>()
                .join("\n");
            response.push_str(&format!("```sql\n{body}\n```"));
        }

        // `response or "Jail's empty!"`
        Ok(if response.is_empty() {
            "Jail's empty!".to_string()
        } else {
            response
        })
    })();

    if let Ok(text) = built {
        respond(ctx, message, text).await;
    }
}

pub async fn who(ctx: &Context, message: &Message) {
    if let Some(guild_id) = message.guild_id {
        // Snapshot cached members (discord.py reads `guild.members`).
        let members: Vec<(i64, String)> = ctx
            .cache
            .guild(guild_id)
            .map(|g| {
                g.members
                    .values()
                    .map(|m| (m.user.id.get() as i64, m.display_name().to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // The whole loop is wrapped in try/except in the original; since the `users`
        // table doesn't exist the first INSERT raises and the rest are skipped, the
        // error is printed, and a ✅ is still added below.
        let _ = (|| -> rusqlite::Result<()> {
            let conn = db::connect()?;
            for (id, name) in &members {
                conn.execute(
                    "INSERT INTO users(id, name) VALUES(?, ?) ON CONFLICT(id) DO UPDATE SET name=?",
                    rusqlite::params![id, name, name],
                )?;
            }
            Ok(())
        })();
    }

    let _ = message
        .react(&ctx.http, discord_util::unicode("✅"))
        .await;
}

#[cfg(test)]
mod tests {
    use super::{py_float_repr, py_str_repr, python_timedelta_str};

    /// Reference values produced by CPython `str(timedelta)`.
    #[test]
    fn timedelta_matches_python() {
        let cases: [(i64, &str); 11] = [
            (0, "0:00:00"),
            (1_000_000, "0:00:01"),
            (59_000_000, "0:00:59"),
            (61_000_000, "0:01:01"),
            (3_661_000_000, "1:01:01"),
            (90_061_000_000, "1 day, 1:01:01"),
            (172_805_000_000, "2 days, 0:00:05"),
            (-1_000_000, "-1 day, 23:59:59"),
            (-86_395_000_000, "-1 day, 0:00:05"),
            (1_799_500_000, "0:29:59.500000"),
            (90_000_000_000, "1 day, 1:00:00"),
        ];
        for (us, expected) in cases {
            assert_eq!(
                python_timedelta_str(chrono::Duration::microseconds(us)),
                expected,
                "us={us}"
            );
        }
    }

    /// Reference values produced by CPython `repr(str)`.
    #[test]
    fn str_repr_matches_python() {
        assert_eq!(py_str_repr("foo"), "'foo'");
        assert_eq!(py_str_repr("it's"), "\"it's\"");
        assert_eq!(py_str_repr("say \"hi\""), "'say \"hi\"'");
        assert_eq!(py_str_repr("a'b\"c"), "'a\\'b\"c'");
        assert_eq!(py_str_repr("tab\there"), "'tab\\there'");
        assert_eq!(py_str_repr(""), "''");
        assert_eq!(py_str_repr("back\\slash"), "'back\\\\slash'");
    }

    /// Reference values produced by CPython `str(float)`.
    #[test]
    fn float_repr_matches_python() {
        assert_eq!(py_float_repr(3.0), "3.0");
        assert_eq!(py_float_repr(2.75), "2.75");
        assert_eq!(py_float_repr(-2.5), "-2.5");
    }
}
