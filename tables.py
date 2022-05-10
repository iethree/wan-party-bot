import sqlite3

conn = sqlite3.connect("wanparty.db")
conn.row_factory = sqlite3.Row
db = conn.cursor()


def initiate_tables():
    try:
        db.execute("CREATE TABLE IF NOT EXISTS counts(name VARCHAR(128), count INT);")
        db.execute(
            "CREATE TABLE IF NOT EXISTS wanbux(id INT PRIMARY KEY, balance INT NOT NULL);"
        )
        db.execute("CREATE TABLE IF NOT EXISTS naughty_list(id INT, user_id INT);")
        db.execute("CREATE TABLE IF NOT EXISTS quotes(user_id INT, quote TEXT);")
    except:
        pass
