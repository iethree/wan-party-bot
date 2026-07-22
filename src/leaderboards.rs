//! Port of `leaderboards.py`. Reachable only via the never-synced `/leaderboards`
//! command, so it never runs in the live bot; reproduced for completeness.

use crate::discord_util;
use crate::lexical_analysis::{lexical_analysis, UserInfo};
use serenity::all::{ChannelId, Context};
use std::collections::HashMap;

/// `fetch_leaderboards(ctx)` — tally per-username message/word counts over the last
/// 10,000 messages, sort by message count descending, and grade each user.
pub async fn fetch_leaderboards(
    ctx: &Context,
    channel_id: ChannelId,
    channel_display_name: Option<String>,
) -> Vec<String> {
    println!("fetching leaderboards");
    // ponytail: 2000 msgs ≈ 20 API calls ≈ ~15s; the original 10,000 took minutes
    // of rate-limited paging. Bump it back up if the stats feel too shallow.
    let history = discord_util::channel_history(ctx, channel_id, Some(2000)).await;

    let mut users: HashMap<String, UserInfo> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for m in &history {
        let name = m.author.name.clone();
        // `len(msg.content.split(" "))` — split on single space; "" -> [""] -> 1.
        let word_count = m.content.split(' ').count() as i64;
        match users.get_mut(&name) {
            Some(u) => {
                u.message_count += 1;
                u.word_count += word_count;
            }
            None => {
                order.push(name.clone());
                users.insert(
                    name.clone(),
                    UserInfo {
                        name,
                        message_count: 1,
                        word_count,
                    },
                );
            }
        }
    }

    let channel_name = channel_display_name.unwrap_or_else(|| "this channel".to_string());
    let mut responses = vec![format!("**Message stats for \"{channel_name}\"**")];

    // sorted(..., key=message_count, reverse=True) — stable.
    let mut sorted_users: Vec<UserInfo> = order.iter().map(|k| users[k].clone()).collect();
    sorted_users.sort_by_key(|u| std::cmp::Reverse(u.message_count));

    println!("analyzing chat data");
    for user_info in &sorted_users {
        let message_count = user_info.message_count;
        let avg_word_count = user_info.word_count / message_count; // int()
        let grade = lexical_analysis(user_info);
        responses.push(format!(
            "> *{}*: **{}** messages, avg length: **{}** words, :brain: estimate: **{}**",
            user_info.name, message_count, avg_word_count, grade
        ));
    }

    responses
}

/// `get_leaderboards(ctx)` — runs the spinner concurrently and cancels it when the
/// fetch finishes.
pub async fn get_leaderboards(
    ctx: &Context,
    channel_id: ChannelId,
    channel_display_name: Option<String>,
) -> Vec<String> {
    let ctx_clone = ctx.clone();
    let thinking_task = tokio::spawn(async move {
        crate::thinking::thinking(&ctx_clone, channel_id, 120, 2).await;
    });

    let responses = fetch_leaderboards(ctx, channel_id, channel_display_name).await;

    thinking_task.abort();
    responses
}
