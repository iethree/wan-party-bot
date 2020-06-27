import sqlite3
conn = sqlite3.connect('wanparty.db')
conn.row_factory = sqlite3.Row
db = conn.cursor()

db.execute('CREATE TABLE counts(name VARCHAR(128), count INT);')

