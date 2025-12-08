#!/usr/bin/python3
import discord
import os
from client import client
from datetime import datetime, timedelta
from emoji import is_emoji
import math
from message_handler import get_emoji

test_channel_id=1307019075700002913
schedule_channel_id=491257084650717195

# only this many options will be added to the poll
MAX_OPTIONS = 8

options = [
  ["Splitgate", "splitgate"],
  ["Marvel Rivals", "🦸"],
  ["Garfield Kart", "😾"],
  ["Jump Space", "🚀"],
  ["Deep Rock Galactic", "dwarf"],
  ["Valheim", "⛺"],
  ["Warhammer Vermintide 2", "🐀"],
  ["RV There Yet?", "🛞"],
  # library below 👇
  ["Peak", "⛰️"],
  ["Helldivers 2", "helldivers"],
  ["Warhammer 40k Darktide", "🔨"],
  ["Halo 2", "halo2"],
  ["Abiotic Factor", "🧑‍🔬"],
  ["Phasmophobia", "👻"],
  ["Sea of Thieves", "sea_of_thieves"],
  ["Arc Raiders", "🌈"],
  ["Project Zomboid", "🧟"],
  ["Left 4 Dead 2", "🧟‍♀️"],
  ["Lethal Company", "🏢"],
  ["Mario Kart World", "mariokart"],
  ["Rematch", "⚽"],
  ["Fortnite", "fortnite"],
  ["Titanfall 2", "🤖"],
  ["Tiny Tina's Wonderlands", "🤪"],
  ["Gunfire Reborn", "🇨🇳"],
  ["Rocket League", "rocket_league"],
  ["Risk of Rain 2",  "🌧️"],
  ["Overwatch 2", "overwatch"],
  ["Void Crew", "🚀"],
  ["Killer Queen Black", "killerqueen"],
  ["MageQuit", "🧙"],
  ["LOTR: Return to Moria", "⛏️"],
  ["Counter Strike 2", "🔫"],
  ["Streets of Rogue", "🛣️"],
  ["Fall Guys", "fallguys"],
  ["Diablo 3", "😈"],
  ["Elite Dangerous", "elite"],
  ["Rounds", "🔴"],
  ["Core Keeper", "🔵"],
  ["Super Smash Bros Ultimate", "🥊"],
  ["MarioKart 8 Deluxe", "🏎️"],
  ["Age of Empires 2", "🏰"],
  ["Starcraft 2", "starcraft"],
  ["Among Us", "amongus"],
  ["Helldivers 1", "🪂"],
]

async def poll(hours = 50):
    print("sending weekly games poll: ", hours, " hours")

    channel = client.get_channel(schedule_channel_id)

    game_poll = discord.Poll(
        question="🎮 What to play on Tuesday? 🎮",
        duration=timedelta(hours = hours),
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

    for option in options[:MAX_OPTIONS]:
      game_poll.add_answer(text=option[0], emoji=emoji(option[1]))

    await channel.send(poll=game_poll)


def hours_left():
    # Get the current datetime
    now = datetime.now()

    day = 2 # Tuesday
    hour = 18 # 6 PM

    # Calculate the number of days until the next party
    days_until_party = (day - now.weekday()) % 7

    # Determine the target day / time
    next_party = (now + timedelta(days=days_until_party)).replace(hour=hour, minute=0, second=0, microsecond=0)

    # If the target time has already passed today, move to the next party
    if next_party <= now:
        next_party += timedelta(weeks=1)

    # Calculate the time difference in hours
    time_difference = next_party - now
    hours_remaining = time_difference.total_seconds() / 3600

    utc_offset = 7 # adjust for DST

    return math.floor(hours_remaining) + utc_offset
