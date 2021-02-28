from discord.utils import get
import random

def get_emoji(guild, emojiName):
  try:
    return get(guild.emojis, name=emojiName)
  except Exception as e:
    return '🙃'

def sometimes(chance):
  return random.random() < chance

async def respond_to(message):
  content = message.content.lower()

  if 'the way' in content:
    await message.add_reaction(get_emoji(message.guild, 'mando'))

  if 'poop' in content:
    await message.add_reaction('💩')

  if 'shplay' in message.author.display_name.lower():
    sometimes(0.1) and await message.add_reaction('🙄')

  if 'DRG' in content or 'dwarf' in content:
    await message.add_reaction('🪨') # rock
    await message.add_reaction('🥌') # stone

  if 'ps5' in content:
    await message.add_reaction('👎')

  if 'star wars' in content and message.channel.name != 'star-wars':
    await message.add_reaction(get_emoji(message.guild, 'stormtrooper'))

