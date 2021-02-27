#!/usr/bin/python3

import sqlite3
import os
import discord
import messageHandler as mh

conn = sqlite3.connect('/tmp/wanparty.db')
db = conn.cursor()

client = discord.Client()

@client.event
async def on_ready():
  print('we have logged in as {0.user}'.format(client))
  

@client.event
async def on_message(message):
  if message.author == client.user:
    return

  fromName = (message.author.nick or message.author.name or message.author)
  print(fromName + ' : ' + message.content)

  response = mh.respondToMessage(message)
  if response:
    await message.channel.send(response)

client.run(os.getenv('DISCORD_TOKEN'))
