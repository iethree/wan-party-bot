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

  commit = sub.run('git log -1 --pretty=%B'.split(), stdout=sub.PIPE)
  env = sub.run('hostname'.split(), stdout=sub.PIPE)
  status_info = env.stdout.decode('utf-8') + ' | ' + commit.stdout.decode('utf-8')
  await client.change_presence(activity=discord.Game(status_info))


@client.event
async def on_message(message):
  if message.author == client.user:
    return

  print(message.author.display_name + ' : ' + message.content)

  response = await mh.respond_to(message)

  if response:
    await message.channel.send(response)

client.run(os.getenv('DISCORD_TOKEN'))
