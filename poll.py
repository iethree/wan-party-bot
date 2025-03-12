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

options = [ # only top 5 options will be added to the poll
  ["Helldivers 2", "helldivers"],
  ["Void Crew", "🚀"],
  ["Deep Rock Galactic", "dwarf"],
  ["Sea of Thieves", "sea_of_thieves"],
  ["Marvel Rivals", "🦸"],
  ["Killer Queen Black", "killerqueen"],
  ["Abiotic Factor", "🧑‍🔬"],
  ["MageQuit", "🧙"],
  ["Gunfire Reborn", "🇨🇳"],
  ["Overwatch 2", "overwatch"],
  ["Fortnite", "fortnite"],
  ["LOTR: Return to Moria", "⛏️"],
  ["Risk of Rain 2",  "🌧️"],
  ["Splitgate 2", "splitgate"],
  ["Rocket League", "rocket_league"],
  ["Warhammer 40k Darktide", "🔨"],
  ["Warhammer Vermintide 2", "🐀"],
  ["Lethal Company", "🏢"],
  ["Halo 2", "halo2"],
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

async def poll(hours = 50):
    print("sending weekly games poll: ", hours, " hours")

    channel = client.get_channel(schedule_channel_id)

    game_poll = discord.Poll(
        question="🎮 What to play on Sunday? 🎮",
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

    for option in options[:8]:
      game_poll.add_answer(text=option[0], emoji=emoji(option[1]))

    await channel.send(poll=game_poll)


def hours_left():
    # Get the current datetime
    now = datetime.now()

    # Calculate the number of days until the next Sunday
    days_until_sunday = (6 - now.weekday()) % 7

    # Determine the target Sunday at 6 PM
    next_sunday_6pm = (now + timedelta(days=days_until_sunday)).replace(hour=18, minute=0, second=0, microsecond=0)

    # If the target time has already passed today, move to the next Sunday
    if next_sunday_6pm <= now:
        next_sunday_6pm += timedelta(weeks=1)

    # Calculate the time difference in hours
    time_difference = next_sunday_6pm - now
    hours_remaining = time_difference.total_seconds() / 3600

    utc_offset = 7 # adjust for DST

    return math.floor(hours_remaining) + utc_offset
