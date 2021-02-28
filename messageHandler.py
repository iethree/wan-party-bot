from discord.utils import get

async def respondToMessage(client, message):
  name = (message.author.nick or message.author.name or message.author)
  content = message.content.lower()

  if 'the way' in content:
    try:
        for e in client.emojis:
            if e.name == 'mando':
                await message.add_reaction(e)
                break
        else:
            names = ', '.join(e.name for e in client.emojis)
            return f':mando: not in client. names=[{names}]'
    except e:
        return f'error fetching :mando:. {e}'

  if 'poop' in content:
    return ":poop:"
