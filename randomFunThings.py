import random
import json
emojis = {}
jokes = {}
responses = {}

with open('data/emoji.json') as f:
  emojis = json.load(f)

with open('data/jokes.json') as f:
  jokes = json.load(f)
 


def randomEmoji(type):
  return ':'+random.choice(list(emojis[type].keys()))+':'

def randomJoke():
  joke = random.choice(jokes)
  return joke['setup'] + '\n\n ||'+ joke['punchline'] +'||'

# def cannedResponse(msg):
#   return next((x for x in responses if msg in x), False)