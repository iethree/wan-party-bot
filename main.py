#!/usr/bin/python3

from google.cloud import storage
import sqlite3
from flask import Flask
import os
import discord


# get db from cloud storage
storage_client = storage.Client()
bucket = storage_client.get_bucket("wan_party_discord_bot")
storage.Blob('wanparty.db', bucket).download_to_filename('/tmp/wanparty.db')
conn = sqlite3.connect('/tmp/wanparty.db')
db = conn.cursor()

# save to cloud storage
def save_db():
	conn.commit()
	storage.Blob('wanparty.db', bucket).upload_from_filename('/tmp/wanparty.db')
	db.close()
	conn.close()

db.execute('SELECT * FROM counts;')
rows = db.fetchall()
for row in rows:
  print(row)
# get db from cloud storage

save_db()

def discord_bot(request):
  return "hello world, someday I will be a discord bot with persistent data and a ci/cd pipeline"

# Bot biz
app = Flask(__name__)
client = discord.Client()

@app.route('/')
def hello_world():
		return "Hello, world!"

@client.event
async def on_ready():
		print(f'Logged in as {client.user}')

# TODO token should change to be whatever it needs to be in the cloud
token = os.environ['DISCORD_SECRET']
client.run(token)
