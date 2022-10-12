from discord.utils import get
import random
from reaction import *
from jokes import random_joke, hit_the_deck
from dick import get_random_quote
import subprocess as sub
import io
import sys
import contextlib


def sometimes(chance):
    return random.random() < chance


STATIC_REACTIONS = [
    Reaction("poop", "💩"),
    Reaction(["drg", "dwarf"], ["🪨", "🥌"]),  # rock and stone
    Reaction("ps5", "👎"),
    Reaction("how you doin bot?", "👍"),
    MatchingReaction(
        lambda c, m: sometimes(0.02) and "shplay" in m.author.display_name.lower(), "🙄"
    ),
    MatchingReaction(
        lambda c, m: sometimes(0.02) and "ben" in m.author.display_name.lower(), "❤️"
    ),
]


@contextlib.contextmanager
def intercept_stdio():
    stdout = sys.stdout
    stderr = sys.stderr
    sys.stdout = io.StringIO()
    sys.stderr = io.StringIO()
    yield sys.stdout, sys.stderr
    sys.stdout = stdout
    sys.stderr = stderr


def get_emoji(guild, emoji_name):
    try:
        return get(guild.emojis, name=emoji_name)
    except Exception as e:
        return "🙃"


async def respond_to(client, message):
    if message.content.startswith("#!"):
        res = sub.run(message.content.split()[1:], stdout=sub.PIPE, stderr=sub.PIPE)
        return (res.stdout + res.stderr).decode("utf-8")

    if message.content.startswith("##"):
        try:
            with intercept_stdio() as (out, err):
                exec(message.content)
            return out.getvalue() + err.getvalue()
        except Exception as e:
            return str(e)

    content = message.content.lower()

    if all(p in content for p in [str(client.user.id), "what", "think"]):
        async for m in message.channel.history(limit=64):
            if m.id == message.id:
                continue
            if "void" not in m.author.name.lower():
                continue
            return m.content
        return "hmmm"

    reactions = STATIC_REACTIONS + [
        Reaction("the way", get_emoji(message.guild, "mando")),
        Reaction("meta", get_emoji(message.guild, "meta")),
        MatchingReaction(
            lambda c, m: "star wars" in c and m.channel.name != "star-wars",
            get_emoji(message.guild, "stormtrooper"),
        ),
    ]

    responses = [
        ["joke", random_joke()],
        ["yoda", get_random_quote('yoda')],
        [" bot ", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wanbot", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["w4nb0t", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wanb0t", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["w4nbot", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wanbutt", "https://giphy.com/gifs/mad-you-u-Q8t3Mfn7TX7gD6l35d"],
        ["wanbort", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["w*nbot", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wanb*t", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["w*nb*t", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wangboy", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wannbot", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wannbott", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ["wanbott", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        ['deck', hit_the_deck()],
        ['trombone', "https://twitter.com/JacobDJAtkinson/status/1572449169666703360"],
    ]

    for reaction in reactions:
        if reaction.matches(content, message):
            await reaction.apply_to(message)

    for response in responses:
        if response[0] in content:
            await message.channel.send(response[1])
