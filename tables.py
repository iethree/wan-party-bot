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
    except:
        pass
