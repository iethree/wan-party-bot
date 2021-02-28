from discord.utils import get

try:
    mando = get(ctx.message.server.emojis, name="mando")
    mando_error = None
except Exception as e:
    mando_error = e

async def respondToMessage(message):
  content = message.content.lower()

  if 'the way' in content:
    if mando_error:
        return f'error fetching :mando: {mando_error}'
    await message.add_reaction(mando)

  if 'poop' in content:
    return ":poop:"
