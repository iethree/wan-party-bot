//! `/poll-config`: a private panel for editing the weekly games poll.
//!
//! Discord has no drag-and-drop, so the panel uses menus and buttons instead:
//!
//! * **Home**: the games in the poll and one page of the backlog, with menus to
//!   move games out of the poll, move backlog games in, or pick a game to edit,
//!   plus page, "Add game" and "Post poll now" buttons.
//! * **Game**: one game, with Edit (name and emoji form), ⬆️/⬇️ to reorder,
//!   Move to backlog / Add to poll, and Delete (asks first).
//!
//! Everything lives in the `poll_options` table ([`crate::poll_options`]). The
//! panel is ephemeral, so only the person who ran `/poll-config` can use it; the
//! command itself is limited to members with Manage Server (admins can change
//! that in Server Settings → Integrations).
//!
//! State is carried in component custom ids (`pc:<action>:<args>`), so every
//! click re-reads the database: two people editing at once see each other's
//! changes on their next click.

use crate::discord_util;
use crate::poll::{hours_left, is_emoji, poll, SCHEDULE_CHANNEL_ID};
use crate::poll_options::{self as opts, EditError, PollOption, MAX_ACTIVE, MAX_NAME_CHARS};
use once_cell::sync::Lazy;
use rusqlite::Connection;
use serenity::all::{
    ActionRowComponent, ButtonStyle, CommandInteraction, ComponentInteraction,
    ComponentInteractionDataKind, Context, CreateActionRow, CreateButton, CreateCommand,
    CreateEmbed, CreateEmbedFooter, CreateInputText, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateModal, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditInteractionResponse, Emoji, GuildId, InputTextStyle,
    InteractionContext, ModalInteraction, Permissions, ReactionType,
};
use std::collections::{HashMap, HashSet};

pub const COMMAND: &str = "poll-config";

const PREFIX: &str = "pc:";
const MODAL_PREFIX: &str = "pc:m:";
const NAME_INPUT: &str = "name";
const EMOJI_INPUT: &str = "emoji";

/// Backlog games per page. The "edit a game" menu lists the poll (up to 10) plus
/// this page, and a menu holds at most 25 options.
const BACKLOG_PAGE: usize = 15;

const ACCENT: u32 = 0x5865F2;

pub fn definition() -> CreateCommand {
    CreateCommand::new(COMMAND)
        .description("Configure the weekly games poll")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .contexts(vec![InteractionContext::Guild])
}

// ---------------------------------------------------------------------------
// Emoji input
// ---------------------------------------------------------------------------

/// `:shortcode:` → unicode, from `data/emoji.json` (category → name → emoji).
static SHORTCODES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let raw: HashMap<String, HashMap<String, String>> =
        serde_json::from_str(include_str!("../data/emoji.json")).unwrap_or_default();
    raw.into_values().flatten().collect()
});

/// Every unicode emoji in `data/emoji.json`, without variation selectors (U+FE0F)
/// so "⛰" and "⛰️" compare equal.
static KNOWN_UNICODE: Lazy<HashSet<String>> =
    Lazy::new(|| SHORTCODES.values().map(|e| strip_vs(e)).collect());

fn strip_vs(s: &str) -> String {
    s.replace('\u{FE0F}', "")
}

/// Turn what someone typed into the form's emoji box into what's stored: a
/// unicode emoji, a server emoji's name, or empty for none. Accepts a unicode
/// emoji, `:shortcode:`, a server emoji name with or without colons, or a pasted
/// `<:name:id>`. `custom` is the server's custom emoji names.
pub fn parse_emoji(input: &str, custom: &HashSet<String>) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(String::new());
    }

    let find_custom = |name: &str| -> Option<String> {
        if custom.contains(name) {
            return Some(name.to_string());
        }
        custom
            .iter()
            .find(|c| c.eq_ignore_ascii_case(name))
            .cloned()
    };

    // A pasted custom emoji: <:name:id> or <a:name:id>.
    if let Some(inner) = input.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        let name = inner.split(':').nth(1).unwrap_or("");
        return find_custom(name)
            .ok_or_else(|| format!("**{name}** isn't an emoji on this server"));
    }

    if !input.is_ascii() {
        return if KNOWN_UNICODE.contains(&strip_vs(input)) {
            Ok(input.to_string())
        } else {
            Err(format!(
                "`{input}` isn't a single emoji I recognise. Use one emoji, a :shortcode:, \
                 or a server emoji name"
            ))
        };
    }

    let name = input.trim_matches(':');
    if let Some(c) = find_custom(name) {
        return Ok(c);
    }
    if let Some(u) = SHORTCODES.get(&name.to_lowercase()) {
        return Ok(u.clone());
    }
    Err(format!("no server emoji or :shortcode: called `{name}`"))
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

type EmojiMap = HashMap<String, Emoji>;

/// How a stored emoji shows up in text: unicode as is, a server emoji as
/// `<:name:id>`, a server emoji that's since been deleted as `:name:`.
fn emoji_text(emoji: &str, map: &EmojiMap) -> String {
    if emoji.is_empty() || is_emoji(emoji) {
        return emoji.to_string();
    }
    match map.get(emoji) {
        Some(e) => e.to_string(),
        None => format!(":{emoji}:"),
    }
}

fn emoji_reaction(emoji: &str, map: &EmojiMap) -> Option<ReactionType> {
    if emoji.is_empty() {
        None
    } else if is_emoji(emoji) {
        Some(ReactionType::Unicode(emoji.to_string()))
    } else {
        map.get(emoji).map(discord_util::emoji_to_reaction)
    }
}

fn label(opt: &PollOption, map: &EmojiMap) -> String {
    let e = emoji_text(&opt.emoji, map);
    if e.is_empty() {
        opt.name.clone()
    } else {
        format!("{e} {}", opt.name)
    }
}

fn select_option(opt: &PollOption, map: &EmojiMap) -> CreateSelectMenuOption {
    let mut o = CreateSelectMenuOption::new(&opt.name, opt.id.to_string());
    if let Some(r) = emoji_reaction(&opt.emoji, map) {
        o = o.emoji(r);
    }
    o
}

/// A menu with `options`, or a disabled one showing `empty` when there are none
/// (Discord requires at least one option even on a disabled menu).
fn menu(
    id: String,
    placeholder: &str,
    options: Vec<CreateSelectMenuOption>,
    max: usize,
    empty: &str,
) -> CreateActionRow {
    let usable = !options.is_empty() && max > 0;
    let options = if options.is_empty() {
        vec![CreateSelectMenuOption::new(empty, "none")]
    } else {
        options
    };
    let max = max.clamp(1, options.len()) as u8;
    CreateActionRow::SelectMenu(
        CreateSelectMenu::new(id, CreateSelectMenuKind::String { options })
            .placeholder(if usable { placeholder } else { empty })
            .min_values(1)
            .max_values(max)
            .disabled(!usable),
    )
}

fn button(id: String, text: &str, style: ButtonStyle) -> CreateButton {
    CreateButton::new(id).label(text).style(style)
}

enum View {
    Home { page: usize },
    Game { id: i64, page: usize },
    ConfirmDelete { id: i64, page: usize },
    ConfirmPost { page: usize },
}

struct Panel {
    content: String,
    embeds: Vec<CreateEmbed>,
    components: Vec<CreateActionRow>,
}

impl Panel {
    fn message(self) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new()
            .content(self.content)
            .embeds(self.embeds)
            .components(self.components)
    }

    fn edit(self) -> EditInteractionResponse {
        EditInteractionResponse::new()
            .content(self.content)
            .embeds(self.embeds)
            .components(self.components)
    }
}

fn pages(backlog_len: usize) -> usize {
    backlog_len.div_ceil(BACKLOG_PAGE).max(1)
}

fn active_embed(active: &[PollOption], map: &EmojiMap) -> CreateEmbed {
    let body = if active.is_empty() {
        "_Nothing yet. Add games from the backlog below._".to_string()
    } else {
        active
            .iter()
            .enumerate()
            .map(|(i, o)| format!("`{:>2}.` {}", i + 1, label(o, map)))
            .collect::<Vec<_>>()
            .join("\n")
    };
    CreateEmbed::new()
        .title(format!("🎮 In the poll ({}/{MAX_ACTIVE})", active.len()))
        .description(body)
        .colour(ACCENT)
}

fn render(conn: &Connection, map: &EmojiMap, view: View, notice: Option<String>) -> Panel {
    let content = notice.unwrap_or_default();
    let (active, backlog) = match (opts::active(conn), opts::backlog(conn)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            return Panel {
                content: format!("❌ database error: {e}"),
                embeds: vec![],
                components: vec![],
            }
        }
    };

    match view {
        View::Home { page } => render_home(content, map, &active, &backlog, page),
        View::ConfirmPost { page } => {
            let hours = hours_left();
            Panel {
                content: format!(
                    "Post this poll to <#{SCHEDULE_CHANNEL_ID}> now? It will close in {hours} hours."
                ),
                embeds: vec![active_embed(&active, map)],
                components: vec![CreateActionRow::Buttons(vec![
                    button(format!("{PREFIX}post!:{page}"), "📮 Post it", ButtonStyle::Primary)
                        .disabled(active.is_empty()),
                    button(format!("{PREFIX}home:{page}"), "Cancel", ButtonStyle::Secondary),
                ])],
            }
        }
        View::Game { id, page } | View::ConfirmDelete { id, page } => {
            let Some(opt) = active.iter().chain(&backlog).find(|o| o.id == id) else {
                return render_home(
                    "⚠️ that game no longer exists".to_string(),
                    map,
                    &active,
                    &backlog,
                    page,
                );
            };
            if matches!(view, View::ConfirmDelete { .. }) {
                return Panel {
                    content: format!("Delete **{}**? This can't be undone.", opt.name),
                    embeds: vec![],
                    components: vec![CreateActionRow::Buttons(vec![
                        button(
                            format!("{PREFIX}del!:{id}:{page}"),
                            "🗑️ Delete",
                            ButtonStyle::Danger,
                        ),
                        button(
                            format!("{PREFIX}game:{id}:{page}"),
                            "Cancel",
                            ButtonStyle::Secondary,
                        ),
                    ])],
                };
            }
            render_game(content, map, &active, opt, page)
        }
    }
}

fn render_home(
    content: String,
    map: &EmojiMap,
    active: &[PollOption],
    backlog: &[PollOption],
    page: usize,
) -> Panel {
    let page_count = pages(backlog.len());
    let page = page.min(page_count - 1);
    let shown = &backlog
        [(page * BACKLOG_PAGE).min(backlog.len())..((page + 1) * BACKLOG_PAGE).min(backlog.len())];

    let backlog_body = if shown.is_empty() {
        "_Empty._".to_string()
    } else {
        shown
            .iter()
            .map(|o| label(o, map))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let backlog_embed = CreateEmbed::new()
        .title(format!("📚 Backlog ({})", backlog.len()))
        .description(backlog_body)
        .footer(CreateEmbedFooter::new(format!(
            "Page {}/{page_count}. The menus below show this page.",
            page + 1
        )));

    let free = MAX_ACTIVE.saturating_sub(active.len());
    let out_menu = menu(
        format!("{PREFIX}out:{page}"),
        "Move games to the backlog…",
        active.iter().map(|o| select_option(o, map)).collect(),
        active.len(),
        "Nothing in the poll to move out",
    );
    let in_menu = menu(
        format!("{PREFIX}in:{page}"),
        &format!("Add backlog games to the poll ({free} free)…"),
        shown.iter().map(|o| select_option(o, map)).collect(),
        free,
        if free == 0 {
            "The poll is full: move a game out first"
        } else {
            "Nothing on this backlog page"
        },
    );
    let pick_menu = menu(
        format!("{PREFIX}pick:{page}"),
        "Edit, reorder or delete a game…",
        active
            .iter()
            .enumerate()
            .map(|(i, o)| select_option(o, map).description(format!("In the poll, #{}", i + 1)))
            .chain(
                shown
                    .iter()
                    .map(|o| select_option(o, map).description("Backlog")),
            )
            .collect(),
        1,
        "No games yet: add one",
    );
    let buttons = CreateActionRow::Buttons(vec![
        button(
            format!("{PREFIX}prev:{page}"),
            "◀ Backlog",
            ButtonStyle::Secondary,
        )
        .disabled(page == 0),
        button(
            format!("{PREFIX}next:{page}"),
            "Backlog ▶",
            ButtonStyle::Secondary,
        )
        .disabled(page + 1 >= page_count),
        button(
            format!("{PREFIX}add:{page}"),
            "➕ Add game",
            ButtonStyle::Success,
        ),
        button(
            format!("{PREFIX}post:{page}"),
            "📮 Post poll now",
            ButtonStyle::Primary,
        )
        .disabled(active.is_empty()),
    ]);

    Panel {
        content,
        embeds: vec![active_embed(active, map), backlog_embed],
        components: vec![out_menu, in_menu, pick_menu, buttons],
    }
}

fn render_game(
    content: String,
    map: &EmojiMap,
    active: &[PollOption],
    opt: &PollOption,
    page: usize,
) -> Panel {
    let id = opt.id;
    let position = active.iter().position(|o| o.id == id);
    let status = match position {
        Some(i) => format!("In the poll, #{} of {}", i + 1, active.len()),
        None => "In the backlog".to_string(),
    };
    let emoji_line = if opt.emoji.is_empty() {
        "No emoji".to_string()
    } else if is_emoji(&opt.emoji) {
        format!("Emoji: {}", opt.emoji)
    } else if map.contains_key(&opt.emoji) {
        format!(
            "Emoji: {} (server emoji `{}`)",
            emoji_text(&opt.emoji, map),
            opt.emoji
        )
    } else {
        format!(
            "Emoji: `{}`, a server emoji that no longer exists",
            opt.emoji
        )
    };
    let embed = CreateEmbed::new()
        .title(label(opt, map))
        .description(format!("{status}\n{emoji_line}"))
        .colour(ACCENT);

    let full = active.len() >= MAX_ACTIVE;
    let toggle = match position {
        Some(_) => button(
            format!("{PREFIX}toggle:{id}:{page}"),
            "📚 Move to backlog",
            ButtonStyle::Secondary,
        ),
        None => button(
            format!("{PREFIX}toggle:{id}:{page}"),
            if full {
                "Poll is full"
            } else {
                "🎮 Add to poll"
            },
            ButtonStyle::Success,
        )
        .disabled(full),
    };
    let row = CreateActionRow::Buttons(vec![
        button(
            format!("{PREFIX}edit:{id}:{page}"),
            "✏️ Edit",
            ButtonStyle::Primary,
        ),
        button(
            format!("{PREFIX}up:{id}:{page}"),
            "⬆️",
            ButtonStyle::Secondary,
        )
        .disabled(!matches!(position, Some(i) if i > 0)),
        button(
            format!("{PREFIX}down:{id}:{page}"),
            "⬇️",
            ButtonStyle::Secondary,
        )
        .disabled(!matches!(position, Some(i) if i + 1 < active.len())),
        toggle,
        button(
            format!("{PREFIX}del:{id}:{page}"),
            "🗑️ Delete",
            ButtonStyle::Danger,
        ),
    ]);
    let back = CreateActionRow::Buttons(vec![button(
        format!("{PREFIX}home:{page}"),
        "↩ Back",
        ButtonStyle::Secondary,
    )]);

    Panel {
        content,
        embeds: vec![embed],
        components: vec![row, back],
    }
}

fn game_modal(custom_id: String, title: &str, existing: Option<&PollOption>) -> CreateModal {
    let mut name = CreateInputText::new(InputTextStyle::Short, "Game", NAME_INPUT)
        .max_length(MAX_NAME_CHARS as u16)
        .required(true);
    let mut emoji = CreateInputText::new(InputTextStyle::Short, "Emoji", EMOJI_INPUT)
        .placeholder("🎮, :rocket:, or a server emoji name like dwarf")
        .max_length(100)
        .required(false);
    if let Some(o) = existing {
        name = name.value(&o.name);
        if !o.emoji.is_empty() {
            emoji = emoji.value(&o.emoji);
        }
    }
    CreateModal::new(custom_id, title).components(vec![
        CreateActionRow::InputText(name),
        CreateActionRow::InputText(emoji),
    ])
}

// ---------------------------------------------------------------------------
// Interaction handlers
// ---------------------------------------------------------------------------

fn emoji_map(ctx: &Context, guild_id: Option<GuildId>) -> EmojiMap {
    guild_id
        .map(|g| discord_util::guild_emoji_map(ctx, g))
        .unwrap_or_default()
}

fn db_error_panel(e: rusqlite::Error) -> Panel {
    Panel {
        content: format!("❌ database error: {e}"),
        embeds: vec![],
        components: vec![],
    }
}

/// Open the panel for `/poll-config`.
pub async fn open(ctx: &Context, cmd: &CommandInteraction) {
    let map = emoji_map(ctx, cmd.guild_id);
    let panel = match opts::connect() {
        Ok(conn) => render(&conn, &map, View::Home { page: 0 }, None),
        Err(e) => db_error_panel(e),
    };
    let response = CreateInteractionResponse::Message(panel.message().ephemeral(true));
    if let Err(e) = cmd.create_response(&ctx.http, response).await {
        println!("error opening poll config: {e}");
    }
}

fn names_of(conn: &Connection, ids: &[i64]) -> String {
    let names: Vec<String> = ids
        .iter()
        .filter_map(|&id| opts::get(conn, id).ok().flatten())
        .map(|o| format!("**{}**", o.name))
        .collect();
    names.join(", ")
}

fn outcome(result: Result<(), EditError>, ok: String) -> Option<String> {
    Some(match result {
        Ok(()) => format!("✅ {ok}"),
        Err(e) => format!("⚠️ {e}"),
    })
}

/// Handle a click on the panel. Returns `false` for components that aren't ours.
pub async fn handle_component(ctx: &Context, comp: &ComponentInteraction) -> bool {
    let Some(rest) = comp.data.custom_id.strip_prefix(PREFIX) else {
        return false;
    };
    let parts: Vec<&str> = rest.split(':').collect();
    let action = parts[0];
    let num = |i: usize| {
        parts
            .get(i)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let values: Vec<i64> = match &comp.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => {
            values.iter().filter_map(|v| v.parse().ok()).collect()
        }
        _ => vec![],
    };
    let map = emoji_map(ctx, comp.guild_id);

    let conn = match opts::connect() {
        Ok(c) => c,
        Err(e) => {
            let msg = db_error_panel(e).message();
            let _ = comp
                .create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(msg))
                .await;
            return true;
        }
    };

    // Actions that open a form, or take long enough to need deferring.
    match action {
        "add" => {
            let modal = game_modal(format!("{MODAL_PREFIX}add:{}", num(1)), "Add a game", None);
            let _ = comp
                .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
                .await;
            return true;
        }
        "edit" => {
            let (id, page) = (num(1), num(2));
            let existing = opts::get(&conn, id).ok().flatten();
            let response = match existing {
                Some(o) => CreateInteractionResponse::Modal(game_modal(
                    format!("{MODAL_PREFIX}edit:{id}:{page}"),
                    "Edit game",
                    Some(&o),
                )),
                None => CreateInteractionResponse::UpdateMessage(
                    render(
                        &conn,
                        &map,
                        View::Home {
                            page: page as usize,
                        },
                        None,
                    )
                    .message(),
                ),
            };
            let _ = comp.create_response(&ctx.http, response).await;
            return true;
        }
        "post!" => {
            let page = num(1) as usize;
            let _ = comp
                .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                .await;
            let notice = match poll(ctx, hours_left()).await {
                Ok(()) => format!("✅ Posted the poll to <#{SCHEDULE_CHANNEL_ID}>"),
                Err(e) => {
                    println!("game poll failed: {e}");
                    format!("❌ {e}")
                }
            };
            let panel = render(&conn, &map, View::Home { page }, Some(notice));
            if let Err(e) = comp.edit_response(&ctx.http, panel.edit()).await {
                println!("error updating poll config: {e}");
            }
            return true;
        }
        _ => {}
    }

    let (view, notice) = match action {
        "out" => {
            let names = names_of(&conn, &values);
            let r = opts::deactivate(&conn, &values);
            let page = num(1) as usize;
            (
                View::Home { page },
                outcome(r, format!("Moved {names} to the backlog")),
            )
        }
        "in" => {
            let names = names_of(&conn, &values);
            let r = opts::activate(&conn, &values);
            let page = num(1) as usize;
            (
                View::Home { page },
                outcome(r, format!("Added {names} to the poll")),
            )
        }
        "pick" => match values.first() {
            Some(&id) => (
                View::Game {
                    id,
                    page: num(1) as usize,
                },
                None,
            ),
            None => (
                View::Home {
                    page: num(1) as usize,
                },
                None,
            ),
        },
        "prev" => (
            View::Home {
                page: (num(1) as usize).saturating_sub(1),
            },
            None,
        ),
        "next" => (
            View::Home {
                page: num(1) as usize + 1,
            },
            None,
        ),
        "home" => (
            View::Home {
                page: num(1) as usize,
            },
            None,
        ),
        "post" => (
            View::ConfirmPost {
                page: num(1) as usize,
            },
            None,
        ),
        "game" => (
            View::Game {
                id: num(1),
                page: num(2) as usize,
            },
            None,
        ),
        "up" | "down" => {
            let (id, page) = (num(1), num(2) as usize);
            let r = opts::shift(&conn, id, if action == "up" { -1 } else { 1 });
            (View::Game { id, page }, r.err().map(|e| format!("⚠️ {e}")))
        }
        "toggle" => {
            let (id, page) = (num(1), num(2) as usize);
            let names = names_of(&conn, &[id]);
            let notice = match opts::get(&conn, id) {
                Ok(Some(o)) if o.active => outcome(
                    opts::deactivate(&conn, &[id]),
                    format!("Moved {names} to the backlog"),
                ),
                Ok(Some(_)) => outcome(
                    opts::activate(&conn, &[id]),
                    format!("Added {names} to the poll"),
                ),
                Ok(None) => Some(format!("⚠️ {}", EditError::NotFound)),
                Err(e) => Some(format!("⚠️ {}", EditError::Db(e))),
            };
            (View::Game { id, page }, notice)
        }
        "del" => (
            View::ConfirmDelete {
                id: num(1),
                page: num(2) as usize,
            },
            None,
        ),
        "del!" => {
            let (id, page) = (num(1), num(2) as usize);
            let names = names_of(&conn, &[id]);
            let r = opts::delete(&conn, id);
            (View::Home { page }, outcome(r, format!("Deleted {names}")))
        }
        _ => (View::Home { page: 0 }, None),
    };

    let msg = render(&conn, &map, view, notice).message();
    if let Err(e) = comp
        .create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(msg))
        .await
    {
        println!("error updating poll config: {e}");
    }
    true
}

fn input_value(modal: &ModalInteraction, id: &str) -> String {
    modal
        .data
        .components
        .iter()
        .flat_map(|row| row.components.iter())
        .find_map(|c| match c {
            ActionRowComponent::InputText(t) if t.custom_id == id => t.value.clone(),
            _ => None,
        })
        .unwrap_or_default()
}

/// Handle a submitted add/edit form. Returns `false` for forms that aren't ours.
pub async fn handle_modal(ctx: &Context, modal: &ModalInteraction) -> bool {
    let Some(rest) = modal.data.custom_id.strip_prefix(MODAL_PREFIX) else {
        return false;
    };
    let parts: Vec<&str> = rest.split(':').collect();
    let num = |i: usize| {
        parts
            .get(i)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let name = input_value(modal, NAME_INPUT);
    let emoji_input = input_value(modal, EMOJI_INPUT);
    let map = emoji_map(ctx, modal.guild_id);
    let custom: HashSet<String> = map.keys().cloned().collect();

    let panel = match opts::connect() {
        Err(e) => db_error_panel(e),
        Ok(conn) => {
            let emoji = parse_emoji(&emoji_input, &custom);
            let (view, notice) = match (parts[0], emoji) {
                ("add", Err(e)) => (
                    View::Home {
                        page: num(1) as usize,
                    },
                    format!("⚠️ didn't add **{}**: {e}", name.trim()),
                ),
                ("add", Ok(emoji)) => {
                    let page = num(1) as usize;
                    match opts::add(&conn, &name, &emoji) {
                        Ok(o) => (
                            View::Game { id: o.id, page },
                            format!(
                                "✅ Added **{}** to {}",
                                o.name,
                                if o.active {
                                    "the poll"
                                } else {
                                    "the backlog (the poll is full)"
                                }
                            ),
                        ),
                        Err(e) => (View::Home { page }, format!("⚠️ {e}")),
                    }
                }
                ("edit", emoji) => {
                    let (id, page) = (num(1), num(2) as usize);
                    let notice = match emoji {
                        Err(e) => format!("⚠️ didn't save: {e}"),
                        Ok(emoji) => match opts::update(&conn, id, &name, &emoji) {
                            Ok(()) => "✅ Saved".to_string(),
                            Err(e) => format!("⚠️ {e}"),
                        },
                    };
                    (View::Game { id, page }, notice)
                }
                _ => (View::Home { page: 0 }, String::new()),
            };
            render(&conn, &map, view, Some(notice))
        }
    };

    let response = CreateInteractionResponse::UpdateMessage(panel.message());
    if let Err(e) = modal.create_response(&ctx.http, response).await {
        println!("error updating poll config: {e}");
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom() -> HashSet<String> {
        ["dwarf", "Splitgate"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn empty_emoji_means_none() {
        assert_eq!(parse_emoji("  ", &custom()), Ok(String::new()));
    }

    #[test]
    fn unicode_emoji_is_kept() {
        assert_eq!(parse_emoji("🚀", &custom()), Ok("🚀".to_string()));
        assert_eq!(parse_emoji("🧟‍♀️", &custom()), Ok("🧟‍♀️".to_string()));
        // With or without the variation selector.
        assert_eq!(parse_emoji("⛰", &custom()), Ok("⛰".to_string()));
    }

    #[test]
    fn text_that_isnt_an_emoji_is_refused() {
        assert!(parse_emoji("é", &custom()).is_err());
        assert!(parse_emoji("🚀🚀", &custom()).is_err());
        assert!(parse_emoji("nope", &custom()).is_err());
    }

    #[test]
    fn shortcodes_become_unicode() {
        assert_eq!(parse_emoji(":rocket:", &custom()), Ok("🚀".to_string()));
        assert_eq!(parse_emoji("rocket", &custom()), Ok("🚀".to_string()));
    }

    #[test]
    fn server_emoji_by_name_or_paste() {
        assert_eq!(parse_emoji("dwarf", &custom()), Ok("dwarf".to_string()));
        assert_eq!(parse_emoji(":dwarf:", &custom()), Ok("dwarf".to_string()));
        assert_eq!(
            parse_emoji("splitgate", &custom()),
            Ok("Splitgate".to_string())
        );
        assert_eq!(
            parse_emoji("<:dwarf:123>", &custom()),
            Ok("dwarf".to_string())
        );
        assert_eq!(
            parse_emoji("<a:dwarf:123>", &custom()),
            Ok("dwarf".to_string())
        );
        assert!(parse_emoji("<:elsewhere:123>", &custom()).is_err());
    }

    #[test]
    fn every_seed_emoji_parses() {
        // The hardcoded options must survive a round trip through the edit form.
        let seeds: HashSet<String> = crate::poll::OPTIONS
            .iter()
            .filter(|(_, e)| e.is_ascii())
            .map(|(_, e)| e.to_string())
            .collect();
        for (_, e) in crate::poll::OPTIONS.iter() {
            assert_eq!(parse_emoji(e, &seeds).as_deref(), Ok(*e), "{e}");
        }
    }

    #[test]
    fn custom_ids_fit_discords_limit() {
        let id = format!("{MODAL_PREFIX}edit:{}:{}", i64::MAX, usize::MAX);
        assert!(id.len() <= 100);
    }

    /// Every view of the seeded table stays inside Discord's component limits,
    /// which Discord enforces by rejecting the whole response.
    #[test]
    fn panels_fit_discords_limits() {
        let conn = Connection::open_in_memory().unwrap();
        opts::ensure(&conn).unwrap();
        let map = EmojiMap::new();
        let first = opts::active(&conn).unwrap()[0].id;
        let last_backlog = opts::backlog(&conn).unwrap().last().unwrap().id;

        let check = |view: View| {
            let panel = render(&conn, &map, view, Some("note".into()));
            assert!(panel.components.len() <= 5);
            let rows = serde_json::to_value(&panel.components).unwrap();
            let mut ids = HashSet::new();
            for row in rows.as_array().unwrap() {
                let comps = row["components"].as_array().unwrap();
                assert!(!comps.is_empty() && comps.len() <= 5);
                for c in comps {
                    let id = c["custom_id"].as_str().unwrap().to_string();
                    assert!(id.len() <= 100);
                    assert!(ids.insert(id), "duplicate custom id");
                    if let Some(options) = c["options"].as_array() {
                        assert!(!options.is_empty() && options.len() <= 25);
                        let max = c["max_values"].as_u64().unwrap() as usize;
                        assert!(max >= 1 && max <= options.len());
                        let values: HashSet<_> =
                            options.iter().map(|o| o["value"].to_string()).collect();
                        assert_eq!(values.len(), options.len(), "duplicate option value");
                    }
                }
            }
            for e in serde_json::to_value(&panel.embeds).unwrap().as_array().unwrap() {
                assert!(e["description"].as_str().unwrap_or("").chars().count() <= 4096);
            }
        };

        for page in 0..4 {
            check(View::Home { page });
        }
        check(View::Game { id: first, page: 0 });
        check(View::Game { id: last_backlog, page: 2 });
        check(View::ConfirmDelete { id: first, page: 0 });
        check(View::ConfirmPost { page: 0 });

        // Empty poll and empty backlog render too.
        conn.execute("DELETE FROM poll_options", []).unwrap();
        check(View::Home { page: 0 });
    }
}
