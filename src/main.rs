//! Port of `main.py` — the bot entrypoint and gateway event handlers.

use serenity::all::{
    ActivityData, Context, EventHandler, GatewayIntents, Interaction, Message, Ready,
};
use serenity::async_trait;
use serenity::Client;

use wan_party_bot::blacklist::is_blacklisted_channel;
use wan_party_bot::{
    botself, chat, client as client_cfg, commands, db, discord_util, message_handler, quote,
};

fn today_mmdd() -> String {
    chrono::Local::now().format("%m-%d").to_string()
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        botself::set_bot_id(ready.user.id.get());
        println!("we have logged in as {}", ready.user.name);

        // status_info = hostname + " | " + last git commit message (each with the
        // trailing newline from the subprocess output, exactly like Python).
        let commit = std::process::Command::new("git")
            .args(["log", "-1", "--pretty=%B"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        let host = std::process::Command::new("hostname")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        let status_info = format!("{host} | {commit}");

        // await tree.sync()  <-- commented out in the original. Sync only
        // (re-)registers commands with Discord; the commands were synced in the
        // past, so Discord still delivers the interactions and the Python tree
        // dispatched them. See `interaction_create` below.

        ctx.set_activity(Some(ActivityData::playing(status_info)));
    }

    async fn message(&self, ctx: Context, message: Message) {
        // if message.author == client.user: return
        if message.author.id.get() == botself::bot_id() {
            return;
        }

        // is_blacklisted_channel(getattr(message.channel, "name", None))
        let channel_name = discord_util::channel_name(&ctx, message.channel_id).await;
        if is_blacklisted_channel(channel_name.as_deref()) {
            return;
        }

        println!(
            "{}: {}",
            discord_util::display_name(&message),
            message.content
        );

        if message.content.starts_with("/quote") {
            quote::quote(&ctx, &message).await;
            return;
        }
        if message.content.starts_with("/comeback") {
            chat::comeback(&ctx, &message).await;
            return;
        }
        if message.content.starts_with("/as") {
            // content.replace("/as", "").strip() — replaces ALL occurrences.
            let personality = message.content.replace("/as", "");
            let personality = personality.trim();
            chat::respond_as(&ctx, &message, personality).await;
            return;
        }
        if message.content.starts_with("/tldr") {
            chat::tldr(&ctx, &message).await;
            return;
        }
        if message.content.starts_with("/recap") {
            chat::recap(&ctx, &message).await;
            return;
        }

        // str(client.user.id) in content (substring test) OR DM channel
        if message.content.contains(&botself::bot_id_str()) || message.guild_id.is_none() {
            chat::bot_response(&ctx, &message).await;
            return;
        }

        let today = today_mmdd();
        if today == "04-01" {
            // Original: `if today == "04-01" and math.random() < 0.05:` — `math.random`
            // doesn't exist, so on April 1 this raises AttributeError and aborts the
            // handler before `respond_to` runs. Reproduced by returning here, so the
            // bot does nothing further for normal messages on April 1.
            return;
        }

        let response = message_handler::respond_to(&ctx, &message).await;
        if let Some(r) = response {
            if !r.is_empty() {
                let _ = message.channel_id.say(&ctx.http, r).await;
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            println!("{}: /{}", cmd.user.name, cmd.data.name);
            commands::dispatch(&ctx, &cmd).await;
        }
    }

    async fn reaction_add(&self, _ctx: Context, _reaction: serenity::all::Reaction) {
        // on_raw_reaction_add in the original takes the wrong arguments and accesses
        // `reaction.message.content` on a raw payload, so it always errors before
        // doing anything observable. Reproduced as a no-op.
    }
}

#[tokio::main]
async fn main() {
    // main.py calls initiate_tables() at import time.
    db::initiate_tables();

    let token = std::env::var("DISCORD_TOKEN").unwrap_or_default();
    let intents: GatewayIntents = client_cfg::intents();

    let mut client = match Client::builder(&token, intents)
        .event_handler(Handler)
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
