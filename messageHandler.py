from randomFunThings import randomJoke, randomEmoji

def respondToMessage(message):
  name = (message.author.nick or message.author.name or message.author)
  
  if 'weather' in message.content:
    return "why don't you go look outside?"
  elif 'work' in message.content:
    return "you should probably do it yourself"
  elif 'goodbye' in message.content:
    return "adios amigo!"
  elif '🐱' in message.content:
    return "https://giphy.com/gifs/cat-dancing-what-PdKTOwHgOASGY"
  elif 'hello' in message.content:
    return 'Hello ' + name + '!'
  elif 'i want' in message.content:
    return "i will get one for you " + name
  elif 'who is the coolest?' in message.content:
    return 'you are, of course, ' + name
  elif 'what is black and white and red all over' in message.content:
    return 'a newspaper, you idiot!'
  elif 'idiot' in message.content:
    return 'I\'m sorry!'
  elif 'joke' in message.content:
    return randomJoke()
  elif 'exercise' in message.content:
    return 'we should go ' + randomEmoji('activity')
  elif 'outside' in message.content:
    return 'You need a ' + randomEmoji('nature')
  elif 'leave' in message.content:
    return 'would you like to ' + randomEmoji('travel')
  elif 'hungry' in message.content:
    return 'would you like a ' + randomEmoji('food')
  else:
    return randomEmoji('people')              
