import random
import json

with open("data/jokes.json") as f:
    jokes = json.load(f)


def random_joke():
    joke = random.choice(jokes)
    return joke["setup"] + "\n\n ||" + joke["punchline"] + "||"

decks = [
    'https://giphy.com/clips/betplus-bet-plus-the-ms-pat-show-APfllllIzLozTEBwbM',
    'https://media.giphy.com/media/MGP5hVgOpcbaVzrV38/giphy.gif',
    'https://media.giphy.com/media/MGP5hVgOpcbaVzrV38/giphy.gif',
]

def hit_the_deck():
    return 'Did someone say DECK??\n' + random.choice(decks)

donks = [
    'https://i.imgur.com/4OmIo0j.png',
]

def hit_the_donk():
    return 'Did someone say DONK??\n' + random.choice(donks)
