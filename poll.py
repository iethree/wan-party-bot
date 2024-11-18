#!/usr/bin/python3
import discord
import os
from client import client
import datetime
from emoji import is_emoji
from message_handler import get_emoji

test_channel_id=1307019075700002913
schedule_channel_id=491257084650717195

options = [ # only top 10 options will be added to the poll
  ["Abiotic Factor", "🧑‍🔬"],
  ["Overwatch 2 (classic mode)", "overwatch"],
  ["Deep Rock Galactic", "dwarf"],
  ["Helldivers 2", "helldivers"],
  ["Gunfire Reborn", "🇨🇳"],
  ["Risk of Rain 2",  "🌧️"],
  ["Rocket League", "rocket_league"],
  ["Warhammer 40k Darktide", "🔨"],
  ["Warhammer Vermintide 2", "🐀"],
  ["Lethal Company", "🏢"],
  ["Halo 2", "halo2"],
  ["Splitgate", "splitgate"],
  ["Fornite", "fortnite"],
  ["Titanfall 2", "🤖"],
  ["Counter Strike 2", "🔫"],
  ["Streets of Rogue", "🛣️"],
  ["Fall Guys", "fallguys"],
  ["Garfield Kart", "😾"],
  ["Diablo 3", "😈"],
  ["Elite Dangerous", "elite"],
  ["Rounds", "🔴"],
  ["Core Keeper", "🔵"],
  ["Super Smash Bros Ultimate", "🥊"],
  ["MarioKart 7 Deluxe", "🏎️"],
  ["Age of Empires 2", "🏰"],
  ["Starcraft 2", "starcraft"],
  ["Among Us", "amongus"],
  ["Helldivers 1", "🪂"],
]

@client.event
async def on_ready():
    print("we have logged in as {0.user}".format(client))
    # await tree.sync()

    channel = client.get_channel(schedule_channel_id)
    poll = discord.Poll(
        question="🎮 What to play on Sunday? 🎮",
        duration=datetime.timedelta(days=3),
        multiple=True,
    )

    def emoji(name):
      if is_emoji(name):
        return name
      else:
        try:
          return get_emoji(channel.guild, name)
        except Exception as e:
          return "🙃"

    for option in options[:10]:
      poll.add_answer(text=option[0], emoji=emoji(option[1]))

    await channel.send(poll=poll)
    os._exit(0)


client.run(os.getenv("DISCORD_TOKEN"))
