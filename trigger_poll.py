#!/usr/bin/python3
import discord
import os
from client import client
from poll import poll, hours_left

@client.event
async def on_ready():
    print("we have logged in as {0.user}".format(client))
    hours = hours_left()
    await poll(hours)
    os._exit(0)

client.run(os.getenv("DISCORD_TOKEN"))
