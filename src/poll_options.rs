//! The weekly games poll's options, stored in sqlite (`poll_options` table).
//!
//! Each game is either *active* (in the poll, ordered by `position`) or in the
//! backlog (sorted by name). The poll holds at most [`MAX_ACTIVE`] games, the
//! most a Discord poll allows. The table is seeded from [`crate::poll::OPTIONS`]
//! the first time it's created: the first 10 options active, the rest backlog.
//!
//! `emoji` is a unicode emoji, a server custom-emoji *name* (resolved when the
//! poll is posted), or empty for none.

use crate::poll::{MAX_OPTIONS, OPTIONS};
use rusqlite::{params, Connection, OptionalExtension};

/// Discord polls allow 10 answers.
pub const MAX_ACTIVE: usize = MAX_OPTIONS;

/// Discord's limit on a poll answer's text.
pub const MAX_NAME_CHARS: usize = 55;

#[derive(Debug, Clone, PartialEq)]
pub struct PollOption {
    pub id: i64,
    pub name: String,
    pub emoji: String,
    pub active: bool,
}

/// Open the database and make sure the table exists.
pub fn connect() -> rusqlite::Result<Connection> {
    let conn = crate::db::connect()?;
    ensure(&conn)?;
    Ok(conn)
}

/// Create the table if it's missing, seeding it from [`OPTIONS`] only then, so
/// deleting every game doesn't bring the defaults back.
pub fn ensure(conn: &Connection) -> rusqlite::Result<()> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'poll_options'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if exists {
        return Ok(());
    }

    conn.execute_batch(
        "CREATE TABLE poll_options(
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            emoji TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 0,
            position INTEGER NOT NULL DEFAULT 0
        );",
    )?;
    let tx = conn.unchecked_transaction()?;
    for (i, (name, emoji)) in OPTIONS.iter().enumerate() {
        let active = i < MAX_ACTIVE;
        tx.execute(
            "INSERT OR IGNORE INTO poll_options(name, emoji, active, position) VALUES(?, ?, ?, ?)",
            params![name, emoji, active, if active { i as i64 } else { 0 }],
        )?;
    }
    tx.commit()
}

fn read(conn: &Connection, sql: &str) -> rusqlite::Result<Vec<PollOption>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(PollOption {
            id: r.get(0)?,
            name: r.get(1)?,
            emoji: r.get(2)?,
            active: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Games in the poll, in poll order.
pub fn active(conn: &Connection) -> rusqlite::Result<Vec<PollOption>> {
    read(
        conn,
        "SELECT id, name, emoji, active FROM poll_options WHERE active = 1 ORDER BY position, id",
    )
}

/// Games in the backlog, by name.
pub fn backlog(conn: &Connection) -> rusqlite::Result<Vec<PollOption>> {
    read(
        conn,
        "SELECT id, name, emoji, active FROM poll_options WHERE active = 0 ORDER BY name COLLATE NOCASE, id",
    )
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<PollOption>> {
    conn.query_row(
        "SELECT id, name, emoji, active FROM poll_options WHERE id = ?",
        [id],
        |r| {
            Ok(PollOption {
                id: r.get(0)?,
                name: r.get(1)?,
                emoji: r.get(2)?,
                active: r.get(3)?,
            })
        },
    )
    .optional()
}

/// Why a change was refused. `Display` is shown to the person editing.
#[derive(Debug)]
pub enum EditError {
    DuplicateName(String),
    EmptyName,
    NameTooLong,
    PollFull,
    NotFound,
    Db(rusqlite::Error),
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::DuplicateName(n) => write!(f, "there's already a game called **{n}**"),
            EditError::EmptyName => f.write_str("the name can't be empty"),
            EditError::NameTooLong => {
                write!(
                    f,
                    "names can be at most {MAX_NAME_CHARS} characters (Discord's poll limit)"
                )
            }
            EditError::PollFull => write!(
                f,
                "the poll is full ({MAX_ACTIVE}/{MAX_ACTIVE}); move a game to the backlog first"
            ),
            EditError::NotFound => f.write_str("that game no longer exists"),
            EditError::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl From<rusqlite::Error> for EditError {
    fn from(e: rusqlite::Error) -> Self {
        EditError::Db(e)
    }
}

fn check_name(conn: &Connection, name: &str, except_id: Option<i64>) -> Result<(), EditError> {
    if name.is_empty() {
        return Err(EditError::EmptyName);
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(EditError::NameTooLong);
    }
    let clash: Option<String> = conn
        .query_row(
            "SELECT name FROM poll_options WHERE name = ? COLLATE NOCASE AND id != ?",
            params![name, except_id.unwrap_or(-1)],
            |r| r.get(0),
        )
        .optional()?;
    match clash {
        Some(existing) => Err(EditError::DuplicateName(existing)),
        None => Ok(()),
    }
}

fn active_count(conn: &Connection) -> rusqlite::Result<usize> {
    conn.query_row(
        "SELECT COUNT(*) FROM poll_options WHERE active = 1",
        [],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n as usize)
}

fn next_position(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM poll_options WHERE active = 1",
        [],
        |r| r.get(0),
    )
}

/// Add a game: into the poll when there's room, otherwise the backlog. Returns
/// the new game.
pub fn add(conn: &Connection, name: &str, emoji: &str) -> Result<PollOption, EditError> {
    let name = name.trim();
    check_name(conn, name, None)?;
    let active = active_count(conn)? < MAX_ACTIVE;
    let position = if active { next_position(conn)? } else { 0 };
    conn.execute(
        "INSERT INTO poll_options(name, emoji, active, position) VALUES(?, ?, ?, ?)",
        params![name, emoji, active, position],
    )?;
    let id = conn.last_insert_rowid();
    get(conn, id)?.ok_or(EditError::NotFound)
}

/// Rename a game and/or change its emoji.
pub fn update(conn: &Connection, id: i64, name: &str, emoji: &str) -> Result<(), EditError> {
    let name = name.trim();
    check_name(conn, name, Some(id))?;
    let changed = conn.execute(
        "UPDATE poll_options SET name = ?, emoji = ? WHERE id = ?",
        params![name, emoji, id],
    )?;
    if changed == 0 {
        return Err(EditError::NotFound);
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), EditError> {
    let tx = conn.unchecked_transaction()?;
    if tx.execute("DELETE FROM poll_options WHERE id = ?", [id])? == 0 {
        return Err(EditError::NotFound);
    }
    renumber(&tx)?;
    tx.commit()?;
    Ok(())
}

/// Move games into the poll, appended in the order given. All or nothing: if
/// they don't all fit, nothing moves.
pub fn activate(conn: &Connection, ids: &[i64]) -> Result<(), EditError> {
    let tx = conn.unchecked_transaction()?;
    let mut count = active_count(&tx)?;
    for &id in ids {
        let Some(opt) = get(&tx, id)? else {
            return Err(EditError::NotFound);
        };
        if opt.active {
            continue;
        }
        if count >= MAX_ACTIVE {
            return Err(EditError::PollFull);
        }
        let position = next_position(&tx)?;
        tx.execute(
            "UPDATE poll_options SET active = 1, position = ? WHERE id = ?",
            params![position, id],
        )?;
        count += 1;
    }
    tx.commit()?;
    Ok(())
}

/// Move games to the backlog.
pub fn deactivate(conn: &Connection, ids: &[i64]) -> Result<(), EditError> {
    let tx = conn.unchecked_transaction()?;
    for &id in ids {
        tx.execute(
            "UPDATE poll_options SET active = 0, position = 0 WHERE id = ?",
            [id],
        )?;
    }
    renumber(&tx)?;
    tx.commit()?;
    Ok(())
}

/// Move an active game `delta` places in the poll order (negative = earlier),
/// stopping at either end.
pub fn shift(conn: &Connection, id: i64, delta: i64) -> Result<(), EditError> {
    let tx = conn.unchecked_transaction()?;
    let mut order: Vec<i64> = active(&tx)?.into_iter().map(|o| o.id).collect();
    let Some(from) = order.iter().position(|&x| x == id) else {
        return Err(EditError::NotFound);
    };
    let to = (from as i64 + delta).clamp(0, order.len() as i64 - 1) as usize;
    let moved = order.remove(from);
    order.insert(to, moved);
    write_order(&tx, &order)?;
    tx.commit()?;
    Ok(())
}

/// Make active positions 0, 1, 2… in their current order. Callers wrap this in
/// their transaction.
fn renumber(conn: &Connection) -> Result<(), EditError> {
    let order: Vec<i64> = active(conn)?.into_iter().map(|o| o.id).collect();
    write_order(conn, &order)
}

/// Callers wrap this in their transaction.
fn write_order(conn: &Connection, order: &[i64]) -> Result<(), EditError> {
    for (i, id) in order.iter().enumerate() {
        conn.execute(
            "UPDATE poll_options SET position = ? WHERE id = ?",
            params![i as i64, id],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure(&conn).unwrap();
        conn
    }

    fn names(v: Vec<PollOption>) -> Vec<String> {
        v.into_iter().map(|o| o.name).collect()
    }

    fn id_of(conn: &Connection, name: &str) -> i64 {
        conn.query_row("SELECT id FROM poll_options WHERE name = ?", [name], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn seeds_from_the_hardcoded_options() {
        let conn = db();
        let expected: Vec<String> = OPTIONS[..MAX_ACTIVE]
            .iter()
            .map(|(n, _)| n.to_string())
            .collect();
        assert_eq!(names(active(&conn).unwrap()), expected);
        assert_eq!(backlog(&conn).unwrap().len(), OPTIONS.len() - MAX_ACTIVE);
    }

    #[test]
    fn deleting_everything_does_not_reseed() {
        let conn = db();
        conn.execute("DELETE FROM poll_options", []).unwrap();
        ensure(&conn).unwrap();
        assert!(active(&conn).unwrap().is_empty());
    }

    #[test]
    fn add_goes_to_backlog_when_full_and_poll_when_not() {
        let conn = db();
        assert!(!add(&conn, "New Game", "🎮").unwrap().active);
        let first = active(&conn).unwrap()[0].id;
        deactivate(&conn, &[first]).unwrap();
        let added = add(&conn, "Another", "").unwrap();
        assert!(added.active);
        assert_eq!(active(&conn).unwrap().last().unwrap().name, "Another");
    }

    #[test]
    fn duplicate_names_are_refused_case_insensitively() {
        let conn = db();
        assert!(matches!(
            add(&conn, "valheim", ""),
            Err(EditError::DuplicateName(n)) if n == "Valheim"
        ));
        let id = id_of(&conn, "Valheim");
        update(&conn, id, "VALHEIM", "⛺").unwrap();
    }

    #[test]
    fn activate_is_all_or_nothing() {
        let conn = db();
        let first = active(&conn).unwrap()[0].id;
        deactivate(&conn, &[first]).unwrap();
        let two: Vec<i64> = backlog(&conn).unwrap()[..2].iter().map(|o| o.id).collect();
        assert!(matches!(activate(&conn, &two), Err(EditError::PollFull)));
        assert_eq!(active(&conn).unwrap().len(), MAX_ACTIVE - 1);
        activate(&conn, &two[..1]).unwrap();
        assert_eq!(active(&conn).unwrap().len(), MAX_ACTIVE);
    }

    #[test]
    fn shift_reorders_and_clamps() {
        let conn = db();
        let before = names(active(&conn).unwrap());
        let second = id_of(&conn, &before[1]);
        shift(&conn, second, -1).unwrap();
        let after = names(active(&conn).unwrap());
        assert_eq!(after[0], before[1]);
        assert_eq!(after[1], before[0]);
        shift(&conn, second, -5).unwrap();
        assert_eq!(names(active(&conn).unwrap())[0], before[1]);
    }

    #[test]
    fn deactivate_keeps_the_rest_in_order() {
        let conn = db();
        let before = names(active(&conn).unwrap());
        deactivate(&conn, &[id_of(&conn, &before[3])]).unwrap();
        let mut expected = before.clone();
        expected.remove(3);
        assert_eq!(names(active(&conn).unwrap()), expected);
    }

    #[test]
    fn name_limits() {
        let conn = db();
        assert!(matches!(add(&conn, "  ", ""), Err(EditError::EmptyName)));
        let long = "x".repeat(MAX_NAME_CHARS + 1);
        assert!(matches!(add(&conn, &long, ""), Err(EditError::NameTooLong)));
    }
}
