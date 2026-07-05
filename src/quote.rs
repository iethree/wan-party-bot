//! Port of `quote.py` — the live `/quote` text command, which saves the replied-to
//! message into the `quotes` table.

use crate::blacklist::is_blacklisted_channel;
use crate::db;
use crate::discord_util;
use serenity::all::{Context, Message};

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
