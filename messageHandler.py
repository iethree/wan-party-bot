
def respondToMessage(message):
  name = (message.author.nick or message.author.name or message.author)
  content = message.content.lower()
  if 'the way' in content:
    return ':mando:'
  if 'poop' in content:
    return ":poop:"