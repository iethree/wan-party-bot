from discord.utils import get

async def respondToMessage(message):
  content = message.content.lower()

  if 'the way' in content:
    await message.add_reaction(get_emoji(message.guild, 'mando'))

  if 'poop' in content:
    await message.add_reaction('💩')

def get_emoji(guild, emojiName):
  try:
    emoji = get(guild.emojis, name=emojiName)
  except Exception as e:
    emoji = '🙃'
  
  return emoji
