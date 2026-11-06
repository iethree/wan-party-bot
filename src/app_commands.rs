//! Discord application commands for the features that started as text prefixes
//! (`/quote`, `/comeback`, `/as`, `/tldr`, `/recap`, `/xkcd` typed as a reply).
//!
//! A slash command can't be sent as a reply, so the reply-based features become
//! right-click commands instead (Apps menu on a message or user):
//!
//! | text prefix         | application command                          |
//! |---------------------|----------------------------------------------|
//! | `/quote` (reply)    | message → Apps → Quote                       |
//! | `/comeback` (reply) | message → Apps → Comeback                    |
//! | `/as X` (reply)     | message → Apps → Respond as… (asks for X)    |
//! | `/tldr` (reply)     | message → Apps → TL;DR                       |
//! | `/recap` (reply)    | user → Apps → Recap, or `/recap user:`       |
//! | `/xkcd [search]`    | `/xkcd search:`, or message → Apps → Find xkcd |
//!
//! Discord allows 5 message commands per app; all 5 are used here. The text
//! prefixes keep working.
//!
//! Every command here is acknowledged with a private (ephemeral) "thinking…"
//! response straight away, because the AI calls run past Discord's 3 second
//! deadline. Output goes to the channel as before; status goes to the private
//! response (see [`Invoker`]).

use crate::invoke::Invoker;
use crate::{chat, poll_config, quote, xkcd};
use serenity::all::{
    ActionRowComponent, ChannelId, Command, CommandDataOptionValue, CommandInteraction,
    CommandOptionType, CommandType, Context, CreateActionRow, CreateCommand, CreateCommandOption,
    CreateInputText, CreateInteractionResponse, CreateModal, InputTextStyle, MessageId,
    ModalInteraction, ResolvedTarget, User, UserId,
};
use std::sync::atomic::{AtomicBool, Ordering};

const QUOTE: &str = "Quote";
const COMEBACK: &str = "Comeback";
const RESPOND_AS: &str = "Respond as…";
const TLDR: &str = "TL;DR";
const FIND_XKCD: &str = "Find xkcd";
const RECAP_USER: &str = "Recap";
const XKCD: &str = "xkcd";
const RECAP: &str = "recap";

/// Modal custom-id prefix; the rest is `<channel id>:<message id>` of the message
/// to respond to.
const RESPOND_AS_MODAL: &str = "respond_as:";
const PERSONALITY_INPUT: &str = "personality";

static REGISTERED: AtomicBool = AtomicBool::new(false);

fn definitions() -> Vec<CreateCommand> {
    let message_cmd = |name: &str| CreateCommand::new(name).kind(CommandType::Message);
    vec![
        message_cmd(QUOTE),
        message_cmd(COMEBACK),
        message_cmd(RESPOND_AS),
        message_cmd(TLDR),
        message_cmd(FIND_XKCD),
        CreateCommand::new(RECAP_USER).kind(CommandType::User),
        CreateCommand::new(XKCD)
            .description("Find the most relevant xkcd comic")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "search",
                    "Describe the situation, e.g. \"too many competing standards\"",
                )
                .max_length(500)
                .required(true),
            ),
        CreateCommand::new(RECAP)
            .description("Recap a year of someone's messages")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "Whose year to recap")
                    .required(true),
            ),
        poll_config::definition(),
    ]
}

/// Register the commands above with Discord. Global command creation is an
/// upsert by name and type, so the commands from `commands.rs` are untouched.
/// Runs once per process: `ready` fires again on every gateway reconnect, and
/// Discord rate-limits command creation.
pub async fn register(ctx: &Context) {
    if REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    for def in definitions() {
        if let Err(e) = Command::create_global_command(&ctx.http, def).await {
            println!("error registering application command: {e}");
        }
    }
}

/// Handle `cmd` if it's one of the commands above. Returns `false` otherwise.
pub async fn dispatch(ctx: &Context, cmd: &CommandInteraction) -> bool {
    let name = cmd.data.name.as_str();
    if cmd.data.kind == CommandType::ChatInput && name == poll_config::COMMAND {
        poll_config::open(ctx, cmd).await;
        return true;
    }
    let known = matches!(
        (cmd.data.kind, name),
        (
            CommandType::Message,
            QUOTE | COMEBACK | RESPOND_AS | TLDR | FIND_XKCD
        ) | (CommandType::User, RECAP_USER)
            | (CommandType::ChatInput, XKCD | RECAP)
    );
    if !known {
        return false;
    }

    // "Respond as…" needs the personality first, and a form has to be the first
    // response, so it can't be deferred like the others.
    if name == RESPOND_AS {
        if let Some(ResolvedTarget::Message(m)) = cmd.data.target() {
            let modal = respond_as_modal(m.channel_id, m.id);
            let _ = cmd
                .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
                .await;
        }
        return true;
    }

    if let Err(e) = cmd.defer_ephemeral(&ctx.http).await {
        println!("error deferring /{name}: {e}");
        return true;
    }
    let inv = Invoker::interaction(&cmd.token);

    match (cmd.data.target(), name) {
        (Some(ResolvedTarget::Message(m)), QUOTE) => quote::quote_message(ctx, &inv, m).await,
        (Some(ResolvedTarget::Message(m)), COMEBACK) => chat::comeback_to(ctx, &inv, m).await,
        (Some(ResolvedTarget::Message(m)), TLDR) => chat::tldr_to(ctx, &inv, m).await,
        (Some(ResolvedTarget::Message(m)), FIND_XKCD) => {
            let query = xkcd::normalize_query(&m.content);
            xkcd::find_and_post(ctx, &inv, query, Some(m), m.channel_id).await;
        }
        (Some(ResolvedTarget::User(user, member)), RECAP_USER) => {
            let display = member
                .and_then(|m| m.nick.clone())
                .unwrap_or_else(|| user_display(user));
            recap(ctx, &inv, cmd, user.id, display).await;
        }
        (_, XKCD) => {
            let query = cmd
                .data
                .options
                .iter()
                .find(|o| o.name == "search")
                .and_then(|o| o.value.as_str())
                .and_then(xkcd::normalize_query);
            xkcd::find_and_post(ctx, &inv, query, None, cmd.channel_id).await;
        }
        (_, RECAP) => {
            let uid = cmd.data.options.iter().find_map(|o| match o.value {
                CommandDataOptionValue::User(uid) => Some(uid),
                _ => None,
            });
            match uid {
                Some(uid) => {
                    let display = cmd
                        .data
                        .resolved
                        .members
                        .get(&uid)
                        .and_then(|m| m.nick.clone())
                        .or_else(|| cmd.data.resolved.users.get(&uid).map(user_display))
                        .unwrap_or_else(|| uid.to_string());
                    recap(ctx, &inv, cmd, uid, display).await;
                }
                None => inv.react(ctx, "🤷").await,
            }
        }
        _ => inv.react(ctx, "🤷").await,
    }

    inv.finish(ctx).await;
    true
}

/// The name a server member sees, short of a server nickname.
fn user_display(user: &User) -> String {
    user.global_name
        .clone()
        .unwrap_or_else(|| user.name.clone())
}

async fn recap(
    ctx: &Context,
    inv: &Invoker<'_>,
    cmd: &CommandInteraction,
    user_id: UserId,
    display: String,
) {
    let target = chat::RecapTarget {
        user_id,
        display,
        guild_id: cmd.guild_id,
        channel_id: cmd.channel_id,
    };
    chat::recap_user(ctx, inv, &target, None).await;
}

fn respond_as_modal(channel_id: ChannelId, message_id: MessageId) -> CreateModal {
    let input = CreateInputText::new(InputTextStyle::Short, "Personality", PERSONALITY_INPUT)
        .placeholder("a pirate, Shakespeare, a disappointed dad…")
        .max_length(200)
        .required(true);
    CreateModal::new(
        format!("{RESPOND_AS_MODAL}{channel_id}:{message_id}"),
        "Respond as…",
    )
    .components(vec![CreateActionRow::InputText(input)])
}

/// The `(channel, message)` a "Respond as…" form was opened on.
fn parse_respond_as_id(custom_id: &str) -> Option<(ChannelId, MessageId)> {
    let rest = custom_id.strip_prefix(RESPOND_AS_MODAL)?;
    let (channel, message) = rest.split_once(':')?;
    let channel = channel.parse::<u64>().ok().filter(|&id| id != 0)?;
    let message = message.parse::<u64>().ok().filter(|&id| id != 0)?;
    Some((ChannelId::new(channel), MessageId::new(message)))
}

/// Handle a submitted "Respond as…" form. Returns `false` for any other modal.
pub async fn dispatch_modal(ctx: &Context, modal: &ModalInteraction) -> bool {
    let Some((channel_id, message_id)) = parse_respond_as_id(&modal.data.custom_id) else {
        return false;
    };

    let personality = modal
        .data
        .components
        .iter()
        .flat_map(|row| row.components.iter())
        .find_map(|c| match c {
            ActionRowComponent::InputText(t) if t.custom_id == PERSONALITY_INPUT => t.value.clone(),
            _ => None,
        })
        .unwrap_or_default();

    if let Err(e) = modal.defer_ephemeral(&ctx.http).await {
        println!("error deferring respond-as form: {e}");
        return true;
    }
    let inv = Invoker::interaction(&modal.token);

    match channel_id.message(&ctx.http, message_id).await {
        Ok(quoted) => chat::respond_as_to(ctx, &inv, &quoted, personality.trim()).await,
        Err(e) => {
            println!("error fetching message for respond-as: {e}");
            inv.react(ctx, "🤷").await;
        }
    }

    inv.finish(ctx).await;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respond_as_id_round_trips() {
        let modal_id = format!("{RESPOND_AS_MODAL}{}:{}", 123, 456);
        assert_eq!(
            parse_respond_as_id(&modal_id),
            Some((ChannelId::new(123), MessageId::new(456)))
        );
    }

    #[test]
    fn other_modal_ids_are_ignored() {
        assert_eq!(parse_respond_as_id("something_else:1:2"), None);
        assert_eq!(parse_respond_as_id("respond_as:abc:2"), None);
        assert_eq!(parse_respond_as_id("respond_as:0:2"), None);
    }

    #[test]
    fn at_most_five_message_commands() {
        // Discord rejects a sixth message-context command.
        let count = definitions()
            .into_iter()
            .filter(|d| serde_json::to_value(d).unwrap()["type"] == serde_json::json!(3))
            .count();
        assert_eq!(count, 5);
    }
}
