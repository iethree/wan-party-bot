import contextlib
from discord.utils import get
import io
import random
import re
import subprocess as sub
import sys

from dick import get_yoda_quote
from jokes import random_joke, hit_the_deck, hit_the_donk
from reaction import *



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
            if "vοid" not in m.author.name.lower():
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
        ["yoda", get_yoda_quote()],
        [" bot ", "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        [re.compile(r"w[\w*]{0,3}n.{0,2}b[\w*]{0,3}[ty]"), "https://giphy.com/gifs/KRY2oGS7SPvO0"],
        [re.compile(r"d\w*ck"), hit_the_deck()],
        [re.compile(r"d\w*nk"), hit_the_donk()],
        [re.compile(r"g[*\w]{0,2}b[*\w]{0,2}\s*g[*\w]{0,5}r", hit_the_deck()],
        ['trombone', "https://twitter.com/JacobDJAtkinson/status/1572449169666703360"],
        ['slide', "https://twitter.com/JacobDJAtkinson/status/1572449169666703360"],
    ]

    for reaction in reactions:
        if reaction.matches(content, message):
            await reaction.apply_to(message)

    for (trigger, response) in responses:
        re_matches = isinstance(trigger, re.Pattern) and trigger.search(content)
        str_matches = isinstance(trigger, str) and trigger in content
        if re_matches or str_matches:
            await message.channel.send(response)
