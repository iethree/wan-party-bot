#!/usr/bin/python3
import sqlite3
import os
import discord
import message_handler as mh
import subprocess as sub

conn = sqlite3.connect('/tmp/wanparty.db')
db = conn.cursor()

client = discord.Client()

@client.event
async def on_ready():
  print('we have logged in as {0.user}'.format(client))
  for c in client.get_all_channels():
      if c.name == 'devs':
          res = sub.run('git log -1'.split(), stdout=sub.PIPE)
          await c.send('I live! ' + res.stdout.decode('utf-8'))


@client.event
async def on_message(message):
  if message.author == client.user:
    return

  print(message.author.display_name + ' : ' + message.content)
  
  response = await mh.respond_to(message)

  if response:
    await message.channel.send(response)

client.run(os.getenv('DISCORD_TOKEN'))
