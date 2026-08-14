//! Port of `quote.py` — the live `/quote` text command, which saves the replied-to
//! message into the `quotes` table.
//!
//! Saved quotes also feed long-term memory (`memory::record_quote`). Quotes long
//! predate memory, so `spawn_quote_backfill` folds the historical table in once per
//! database on startup.

use crate::blacklist::is_blacklisted_channel;
use crate::db;
use crate::discord_util;
use crate::memory;
use rusqlite::Connection;
use serenity::all::{Context, Message, UserId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn quote(ctx: &Context, message: &Message) {
    let name = discord_util::channel_name(ctx, message.channel_id).await;
    if is_blacklisted_channel(name.as_deref()) {
        let _ = message
            .react(&ctx.http, discord_util::unicode("🙅‍♀️"))
            .await;
        return;
    }

    // `message.channel.fetch_message(message.reference.message_id)`. Without a
    // reply (no reference), the original raises and sends the apology below.
    let quoted = match message.message_reference.as_ref().and_then(|r| r.message_id) {
        Some(mid) => match message.channel_id.message(&ctx.http, mid).await {
            Ok(m) => m,
            Err(_) => {
                let _ = message
                    .channel_id
                    .say(
                        &ctx.http,
                        "you probably didn't quote something, or the dev was too lazy to handle the error right",
                    )
                    .await;
                return;
            }
        },
        None => {
            let _ = message
                .channel_id
                .say(
                    &ctx.http,
                    "you probably didn't quote something, or the dev was too lazy to handle the error right",
                )
                .await;
            return;
        }
    };

    println!("quoting {}", quoted.content);

    let insert = (|| -> rusqlite::Result<()> {
        let conn = db::connect()?;
        conn.execute(
            "INSERT INTO quotes(user_id, quote) VALUES(?,?)",
            rusqlite::params![quoted.author.id.get() as i64, quoted.content],
        )?;
        Ok(())
    })();

    match insert {
        Ok(()) => {
            // Remember it against the person who said it, not whoever ran /quote.
            memory::record_quote(
                quoted.author.id.get(),
                &discord_util::display_name(&quoted),
                &quoted.content,
            );
            let _ = message
                .react(&ctx.http, discord_util::unicode("✅"))
                .await;
        }
        Err(e) => {
            let _ = message.channel_id.say(&ctx.http, e.to_string()).await;
            let _ = message
                .react(&ctx.http, discord_util::unicode("❌"))
                .await;
        }
    }
}

// ----------------------------------------------------------------------------
// One-time backfill of the historical quotes table into long-term memory
// ----------------------------------------------------------------------------

/// `PRAGMA user_version` value meaning "the quotes backlog has been folded into
/// memory". SQLite's own per-database counter, so no migration bookkeeping table.
const QUOTES_INGESTED_VERSION: i64 = 1;

/// Quotes per summarization call. Kept under memory's own auto-flush threshold so the
/// explicit flush below is the one that fires.
const BACKFILL_CHUNK: usize = 30;

static BACKFILL_STARTED: AtomicBool = AtomicBool::new(false);

/// Fold the historical `quotes` table into long-term memory, once per database.
///
/// Runs in the background: a large backlog is many sequential summarization calls, and
/// blocking `ready` on it would keep the bot offline for the duration.
pub fn spawn_quote_backfill(ctx: Context) {
    // `ready` fires again on every gateway reconnect; only the first one counts.
    if BACKFILL_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    tokio::spawn(backfill_quotes(ctx));
}

async fn backfill_quotes(ctx: Context) {
    match db::connect().and_then(|c| schema_version(&c)) {
        Ok(v) if v >= QUOTES_INGESTED_VERSION => return,
        Ok(_) => {}
        Err(e) => {
            println!("quote backfill: can't read user_version ({e}); skipping");
            return;
        }
    }

    let rows = match db::connect().and_then(|c| read_backlog(&c)) {
        Ok(r) => r,
        Err(e) => {
            println!("quote backfill: can't read quotes ({e}); will retry on next start");
            return;
        }
    };
    if rows.is_empty() {
        mark_ingested();
        return;
    }

    let names = resolve_names(&ctx, &rows).await;
    println!(
        "quote backfill: folding {} historical quote(s) from {} user(s) into memory",
        rows.len(),
        names.len()
    );

    let mut done = 0usize;
    for chunk in rows.chunks(BACKFILL_CHUNK) {
        for (uid, text) in chunk {
            let name = names
                .get(uid)
                .cloned()
                .unwrap_or_else(|| format!("user {uid}"));
            memory::record_quote(*uid as u64, &name, text);
        }
        if !memory::flush_now().await {
            // The failed batch stays queued for the periodic flusher. Stop here rather
            // than pushing the rest at an API that's already failing, and leave the
            // version unset so the next start redoes the whole backlog — the
            // summarizer merges, so folding a chunk twice is harmless.
            println!("quote backfill: stopped after {done} quote(s); will retry on next start");
            return;
        }
        done += chunk.len();
        println!("quote backfill: {done}/{} folded", rows.len());
    }

    mark_ingested();
}

/// Oldest first (rowid is insertion order), skipping empty quotes.
fn read_backlog(conn: &Connection) -> rusqlite::Result<Vec<(i64, String)>> {
    let mut stmt =
        conn.prepare("SELECT user_id, quote FROM quotes WHERE TRIM(quote) <> '' ORDER BY rowid")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// Display name per distinct author, so backfilled sections merge with the ones live
/// interactions create. Falls back to the bare id for users we can't fetch (left the
/// server, deleted account).
async fn resolve_names(ctx: &Context, rows: &[(i64, String)]) -> HashMap<i64, String> {
    let mut names: HashMap<i64, String> = HashMap::new();
    for (uid, _) in rows {
        if names.contains_key(uid) {
            continue;
        }
        // ponytail: sequential — a server's worth of distinct authors, once per
        // database. Batch via guild members if a backlog ever spans thousands.
        if let Ok(user) = UserId::new(*uid as u64).to_user(ctx).await {
            names.insert(*uid, user.display_name().to_string());
        }
    }
    names
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
}

fn set_ingested(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "user_version", QUOTES_INGESTED_VERSION)
}

fn mark_ingested() {
    match db::connect().and_then(|c| set_ingested(&c)) {
        Ok(()) => println!("quote backfill: done"),
        Err(e) => println!("quote backfill: done, but couldn't record it ({e}); it will run again"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backlog_reads_oldest_first_and_records_completion() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE quotes(user_id INT, quote TEXT)", [])
            .unwrap();
        for (uid, quote) in [(7i64, "first"), (9, "   "), (7, "second")] {
            conn.execute(
                "INSERT INTO quotes(user_id, quote) VALUES(?,?)",
                rusqlite::params![uid, quote],
            )
            .unwrap();
        }

        // Insertion order, blank quotes dropped.
        assert_eq!(
            read_backlog(&conn).unwrap(),
            vec![(7, "first".to_string()), (7, "second".to_string())]
        );

        // A fresh database runs the backfill; one that has run it skips.
        assert_eq!(schema_version(&conn).unwrap(), 0);
        set_ingested(&conn).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), QUOTES_INGESTED_VERSION);
    }
}
