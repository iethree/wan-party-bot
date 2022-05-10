#!/usr/bin/python3
import sqlite3
import os
import discord
import message_handler as mh
import subprocess as sub
from commands import bot
from tables import initiate_tables
from datetime import date
from sing import *

initiate_tables()

@bot.event
async def on_ready():
    print("we have logged in as {0.user}".format(bot))

    commit = sub.run("git log -1 --pretty=%B".split(), stdout=sub.PIPE)
    env = sub.run("hostname".split(), stdout=sub.PIPE)
    status_info = env.stdout.decode("utf-8") + " | " + commit.stdout.decode("utf-8")
    await bot.change_presence(activity=discord.Game(status_info))


@bot.event
async def on_message(message):
    if message.author == bot.user:
        return

    print(message.author.display_name + ": " + message.content)

    today = date.today().strftime("%m-%d")

    if (today == "04-01"):
        await message.channel.send(sing_to_me())
        return

    await bot.process_commands(message)

    response = await mh.respond_to(bot, message)

    if response:
        await message.channel.send(response)

bot.run(os.getenv("DISCORD_TOKEN"))
