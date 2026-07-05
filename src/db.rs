//! Port of `tables.py` plus a shared connection helper.
//!
//! Every Python DB call does `sqlite3.connect(DATABASE)` afresh; we mirror that
//! by opening a new connection per operation.

use rusqlite::Connection;

pub const DATABASE: &str = "wanparty.db";

pub fn connect() -> rusqlite::Result<Connection> {
    Connection::open(DATABASE)
}

/// `initiate_tables()` — wrapped in try/except pass in Python, so any error is
/// swallowed.
pub fn initiate_tables() {
    let result = (|| -> rusqlite::Result<()> {
        let conn = Connection::open(DATABASE)?;
        conn.execute("CREATE TABLE IF NOT EXISTS counts(name VARCHAR(128), count INT);", [])?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wanbux(id INT PRIMARY KEY, balance INT NOT NULL);",
            [],
        )?;
        conn.execute("CREATE TABLE IF NOT EXISTS naughty_list(id INT, user_id INT);", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS quotes(user_id INT, quote TEXT);", [])?;
        Ok(())
    })();
    if let Err(e) = result {
        // matches `except: pass`, but log like the rest of the app does
        println!("initiate_tables error: {e}");
    }
}
