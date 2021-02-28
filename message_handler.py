from discord.utils import get
import random
from reaction import *

def sometimes(chance):
  return random.random() < chance

STATIC_REACTIONS = [
    Reaction('poop', '💩'),
    Reaction(['DRG', 'dwarf'], ['🪨', '🥌']), # rock and stone
    Reaction('ps5', '👎'),
    MatchingReaction(lambda c, m: sometimes(0.1) and 'shplay' in m.author.display_name.lower(), '🙄')
]

def get_emoji(guild, emoji_name):
  try:
    return get(guild.emojis, name=emoji_name)
  except Exception as e:
    return '🙃'

async def respond_to(message):
  content = message.content.lower()
  reactions = STATIC_REACTIONS + [
      Reaction('the way', get_emoji(message.guild, 'mando')),
      MatchingReaction(lambda c, m: 'star wars' in c and m.channel.name != 'star-wars', get_emoji(message.guild, 'stormtrooper'))
  ]
  for reaction in reactions:
      if reaction.matches(content, message):
          await reaction.apply_to(message)

