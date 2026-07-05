//! Port of `trigger_poll.py` — connect, post the weekly games poll on READY, then
//! exit the process immediately (`os._exit(0)`).

use serenity::all::{Context, EventHandler, GatewayIntents, Ready};
use serenity::async_trait;
use serenity::Client;

use wan_party_bot::client as client_cfg;
use wan_party_bot::poll::{hours_left, poll};

struct PollHandler;

#[async_trait]
impl EventHandler for PollHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("we have logged in as {}", ready.user.name);
        let hours = hours_left();
        poll(&ctx, hours).await;
        std::process::exit(0);
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").unwrap_or_default();
    let intents: GatewayIntents = client_cfg::intents();

    let mut client = match Client::builder(&token, intents)
        .event_handler(PollHandler)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error creating client: {e}");
            return;
        }
    };

    if let Err(e) = client.start().await {
        eprintln!("client error: {e}");
    }
}
