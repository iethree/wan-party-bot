import random
import json

with open("data/jokes.json") as f:
    jokes = json.load(f)


def random_joke():
    joke = random.choice(jokes)
    return joke["setup"] + "\n\n ||" + joke["punchline"] + "||"
