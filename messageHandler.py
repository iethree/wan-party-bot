
def respondToMessage(message):
  name = (message.author.nick or message.author.name or message.author)
  
  if 'poop' in message.content:
    return ":poop:"
