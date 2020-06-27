from google.cloud import storage
import sqlite3

# get db from cloud storage
storage_client = storage.Client()
bucket = storage_client.get_bucket("wan_party_discord_bot")
storage.Blob('wanparty.db', bucket).download_to_filename('wanparty.db')
conn = sqlite3.connect('wanparty.db')
db = conn.cursor()

# save to cloud storage
def save_db():
  conn.commit()
  storage.Blob('wanparty.db', bucket).upload_from_filename('wanparty.db')
  db.close()
  conn.close()

db.execute('SELECT * FROM counts;')
rows = db.fetchall()
for row in rows:
  print(row)
# get db from cloud storage

save_db()

def discord_bot(request):
  return "hello world, someday I will be a discord bot with persistent data"

