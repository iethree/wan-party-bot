//! `/xkcd [search]` — find the most relevant xkcd comic and post it as an embed.
//!
//! The search text is whatever follows `/xkcd`. With no text, the command must be
//! a reply, and the replied-to message's content is the search text. Searching
//! uses the findxkcd API, which returns results ranked by relevance.

use crate::chat;
use crate::discord_util;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::all::{Context, CreateEmbed, CreateEmbedFooter, CreateMessage, Message};

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

fn normalize_query(text: &str) -> Option<String> {
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

/// Search findxkcd and return the most relevant comic, if any matched.
pub async fn search(query: &str) -> Result<Option<Comic>, String> {
    let body = HTTP
        .get(SEARCH_URL)
        .query(&[("q", query), ("limit", "1")])
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    top_result(&body).map_err(|e| e.to_string())
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

async fn react(ctx: &Context, msg: &Message, emoji: &str) {
    let _ = msg.react(&ctx.http, discord_util::unicode(emoji)).await;
}

/// `/xkcd [search]`. Replies to the replied-to message when used as a reply,
/// otherwise to the command message. Reacts 🤷 when there is nothing to search
/// for or nothing matched, and 😵 when the search fails.
pub async fn xkcd(ctx: &Context, message: &Message) {
    let quoted = chat::get_quoted_msg(ctx, message).await;

    let query = query_from_command(&message.content)
        .or_else(|| quoted.as_ref().and_then(|q| normalize_query(&q.content)));
    let Some(query) = query else {
        react(ctx, message, "🤷").await;
        return;
    };

    println!("searching xkcd for {query}");
    let comic = match search(&query).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            react(ctx, message, "🤷").await;
            return;
        }
        Err(e) => {
            println!("error searching xkcd");
            println!("{e}");
            react(ctx, message, "😵").await;
            return;
        }
    };

    let target = quoted.as_ref().unwrap_or(message);
    let reply = CreateMessage::new()
        .embed(comic_embed(&comic))
        .reference_message(target);
    if let Err(e) = message.channel_id.send_message(&ctx.http, reply).await {
        println!("error sending xkcd");
        println!("{e}");
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
