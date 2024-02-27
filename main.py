#!/usr/bin/python3
import sqlite3
import os
import discord
import message_handler as mh
import subprocess as sub
from tables import initiate_tables
from datetime import date
from sing import *
from client import client
from commands import tree
from quote import quote
from chat import comeback, kindness, bot_response, ted

initiate_tables()


@client.event
async def on_ready():
    print("we have logged in as {0.user}".format(client))

    commit = sub.run("git log -1 --pretty=%B".split(), stdout=sub.PIPE)
    env = sub.run("hostname".split(), stdout=sub.PIPE)
    status_info = env.stdout.decode("utf-8") + " | " + commit.stdout.decode("utf-8")
    # await tree.sync()

    await client.change_presence(activity=discord.Game(status_info))


@client.event
async def on_message(message):
    if message.author == client.user:
        return

    print(message.author.display_name + ": " + message.content)

    if message.content.startswith("/quote"):
        await quote(message)
        return

    if message.content.startswith("/comeback"):
        await comeback(message)
        return

    if message.content.startswith("/kindness"):
        await kindness(message)
        return

    if message.content.startswith("/ted"):
        await ted(message)
        return

    if str(client.user.id) in message.content:
        await bot_response(message)
        return

    today = date.today().strftime("%m-%d")

    if today == "04-01":
        await message.channel.send(sing_to_me())
        return

    # await client.process_commands(message)

    response = await mh.respond_to(client, message)

    if response:
        await message.channel.send(response)

@client.event
async def on_raw_reaction_add(reaction, user):
    print(reaction.message.content)


client.run(os.getenv("DISCORD_TOKEN"))
