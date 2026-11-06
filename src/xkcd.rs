//! xkcd search: find the most relevant xkcd comic and post it as an embed.
//!
//! Three ways in: `/xkcd <search>` as a slash command, message → Apps → Find
//! xkcd (searches that message), or `/xkcd [search]` typed as text, where a bare
//! `/xkcd` sent as a reply searches the replied-to message. Searching uses the
//! findxkcd API, which ranks results by relevance and leaves out weak matches.

use crate::blacklist::is_blacklisted_channel;
use crate::chat;
use crate::discord_util;
use crate::invoke::Invoker;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::all::{ChannelId, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, Message};

const SEARCH_URL: &str = "https://findxkcd.infopanel.org/api/search";

/// Longest search text sent to the API, in chars. A replied-to message can be
/// long; the start of it is enough to find a match and keeps the URL short.
const MAX_QUERY_CHARS: usize = 500;

static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<Comic>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Comic {
    pub num: u32,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub alt: String,
    pub img: String,
    #[serde(default)]
    pub published: Option<String>,
}

/// The text after `/xkcd`, trimmed and capped at [`MAX_QUERY_CHARS`]. `None`
/// when there is no text.
pub fn query_from_command(content: &str) -> Option<String> {
    let rest = content.strip_prefix("/xkcd").unwrap_or(content);
    normalize_query(rest)
}

pub fn normalize_query(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    Some(text.chars().take(MAX_QUERY_CHARS).collect())
}

/// Parse a search response body and return the top result.
fn top_result(body: &str) -> Result<Option<Comic>, serde_json::Error> {
    let resp: SearchResponse = serde_json::from_str(body)?;
    Ok(resp.results.into_iter().next())
}

/// Why a search failed.
#[derive(Debug)]
pub enum SearchError {
    /// 429: findxkcd allows 30 searches per minute per IP.
    RateLimited,
    Other(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::RateLimited => f.write_str("rate limited (429)"),
            SearchError::Other(m) => f.write_str(m),
        }
    }
}

/// Search findxkcd and return the most relevant comic, if any matched. The API
/// leaves out weak matches, so `Ok(None)` is common for vague or quoted text.
pub async fn search(query: &str) -> Result<Option<Comic>, SearchError> {
    let other = |e: reqwest::Error| SearchError::Other(e.to_string());
    let resp = HTTP
        .get(SEARCH_URL)
        .query(&[("q", query), ("limit", "1")])
        .send()
        .await
        .map_err(other)?;
    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(SearchError::RateLimited);
    }
    let body = resp
        .error_for_status()
        .map_err(other)?
        .text()
        .await
        .map_err(other)?;
    top_result(&body).map_err(|e| SearchError::Other(e.to_string()))
}

fn comic_embed(comic: &Comic) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("xkcd #{}: {}", comic.num, comic.title))
        .url(&comic.url)
        .image(&comic.img);
    if !comic.alt.is_empty() {
        // Embed descriptions are capped at 4096 chars; alt text is far shorter in
        // practice, but don't let one outlier fail the send.
        embed = embed.description(comic.alt.chars().take(4096).collect::<String>());
    }
    if let Some(published) = comic.published.as_deref().filter(|p| !p.is_empty()) {
        embed = embed.footer(CreateEmbedFooter::new(published));
    }
    embed
}

/// `/xkcd [search]` typed as text. The search is the text after `/xkcd`, or the
/// replied-to message when there is none.
pub async fn xkcd(ctx: &Context, message: &Message) {
    let quoted = chat::get_quoted_msg(ctx, message).await;
    let query = query_from_command(&message.content)
        .or_else(|| quoted.as_ref().and_then(|q| normalize_query(&q.content)));
    let inv = Invoker::Message(message);
    find_and_post(
        ctx,
        &inv,
        query,
        quoted.as_ref().or(Some(message)),
        message.channel_id,
    )
    .await;
}

/// Search for `query` and post the top comic to `channel_id`, as a reply to
/// `reply_to` when given. Reports 🤷 when there is nothing to search for, a
/// "no xkcd found" note when nothing matched, and 😵 when the search fails.
pub async fn find_and_post(
    ctx: &Context,
    inv: &Invoker<'_>,
    query: Option<String>,
    reply_to: Option<&Message>,
    channel_id: ChannelId,
) {
    let name = discord_util::channel_name(ctx, channel_id).await;
    if is_blacklisted_channel(name.as_deref()) {
        inv.react(ctx, "🙅‍♀️").await;
        return;
    }

    let Some(query) = query else {
        inv.react(ctx, "🤷").await;
        return;
    };

    println!("searching xkcd for {query}");
    let comic = match search(&query).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            println!("no xkcd found for {query}");
            inv.say(ctx, &format!("no xkcd found for \"{}\"", preview(&query)))
                .await;
            return;
        }
        Err(SearchError::RateLimited) => {
            println!("xkcd search rate limited");
            inv.say(ctx, "too many xkcd searches, try again in a minute")
                .await;
            return;
        }
        Err(e) => {
            println!("error searching xkcd");
            println!("{e}");
            inv.react(ctx, "😵").await;
            return;
        }
    };

    let mut post = CreateMessage::new().embed(comic_embed(&comic));
    if let Some(target) = reply_to {
        post = post.reference_message(target);
    }
    if let Err(e) = channel_id.send_message(&ctx.http, post).await {
        println!("error sending xkcd");
        println!("{e}");
        inv.react(ctx, "😵").await;
    }
}

/// The start of a search, for echoing it back without a wall of text.
fn preview(query: &str) -> String {
    const MAX: usize = 80;
    if query.chars().count() <= MAX {
        query.to_string()
    } else {
        format!("{}…", query.chars().take(MAX).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_text_after_command() {
        assert_eq!(
            query_from_command("/xkcd  password strength "),
            Some("password strength".to_string())
        );
    }

    #[test]
    fn bare_command_has_no_query() {
        assert_eq!(query_from_command("/xkcd"), None);
        assert_eq!(query_from_command("/xkcd   \n"), None);
    }

    #[test]
    fn long_query_is_capped() {
        let long = format!("/xkcd {}", "a".repeat(MAX_QUERY_CHARS * 2));
        assert_eq!(
            query_from_command(&long).unwrap().chars().count(),
            MAX_QUERY_CHARS
        );
    }

    #[test]
    fn parses_top_result() {
        let body = r#"{"query":"password strength","results":[{"num":936,"title":"Password Strength","url":"https://xkcd.com/936/","explain_url":"https://www.explainxkcd.com/wiki/index.php/936","alt":"To anyone who understands information theory...","img":"https://imgs.xkcd.com/comics/password_strength.png","published":"2011-08-10","score":1}]}"#;
        let comic = top_result(body).unwrap().unwrap();
        assert_eq!(comic.num, 936);
        assert_eq!(comic.title, "Password Strength");
        assert_eq!(
            comic.img,
            "https://imgs.xkcd.com/comics/password_strength.png"
        );
        assert_eq!(comic.published.as_deref(), Some("2011-08-10"));
    }

    #[test]
    fn empty_results_is_none() {
        assert_eq!(top_result(r#"{"query":"x","results":[]}"#).unwrap(), None);
    }
}
