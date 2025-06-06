import contextlib
from discord.utils import get
import io
import random
import re
import subprocess as sub
import sys
from datetime import date

from dick import get_yoda_quote
from reaction import *

def sometimes(chance):
    return random.random() < chance

did_u_say_wanbot = re.compile(r"\bw[\w*]{0,3}n.{0,2}[8b][\w*]{0,3}[ty]\b")

def did_u_say_the_magic_word(words, message):
    random.shuffle(words)
    return any(word in message for word in words)

STATIC_REACTIONS = [
    Reaction("poop", "💩"),
    Reaction(["drg", "dwarf"], ["🪨", "🥌"]),  # rock and stone
    Reaction("ps5", "👎"),
    Reaction("dongle", "🍆"),
    Reaction("how you doin bot?", "👍"),
    MatchingReaction(
      lambda c, m: sometimes(0.2) and did_u_say_the_magic_word([
          "opinion",
          "take",
          "ps5",
          "valve",
          "steam deck",
          "harry potter",
          "dune",
          "star wars",
          "wow",
          "factorio",
          "the last of us",
          "overwatch",
          "facts",
          "objective",
          "correct"
      ], m.content.lower()), "🙄"
    ),
    MatchingReaction(
        lambda c, m: did_u_say_wanbot.search(c) is not None, "💁‍♀️"
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
        Reaction("texas", get_emoji(message.guild, "happytexas")),
        MatchingReaction(
            lambda c, m: "star wars" in c and m.channel.name != "star-wars",
            get_emoji(message.guild, "stormtrooper"),
        ),
        MatchingReaction(
            lambda content, message: bool(re.search(r"d[eéèë*3!]ck|d[oóòö*0!]nk|g[*\w]{0,2}b[*\w]{0,2}\s*g[*\w]{0,5}r", content)),
            get_emoji(message.guild, "thedeck"),
        ),
        MatchingReaction(
            lambda c, m: "ryan" in m.author.display_name.lower() and ("wow" in c or "warcraft" in c),
            get_emoji(message.guild, "wow"),
        ),
        MatchingReaction(
            lambda c, m: "tsm" in m.author.display_name.lower() and sometimes(0.05),
            get_emoji(message.guild, "neato"),
        ),
        MatchingReaction(
            lambda c, m: "local_oaf" in m.author.name.lower() and sometimes(0.02),
            get_emoji(message.guild, "elon"),
        ),
        MatchingReaction(
            lambda c, m: "shplay" in m.author.display_name.lower() and (date.today().strftime("%m-%d") == "04-20"),
            get_emoji(message.guild, "420shplaybday"),
        ),
    ]

    responses = [
        ["yoda", get_yoda_quote()],
        ['trombone', "https://twitter.com/JacobDJAtkinson/status/1572449169666703360"],
    ]

    for reaction in reactions:
        if reaction.matches(content, message):
            await reaction.apply_to(message)


    for (trigger, response) in responses:
        re_matches = isinstance(trigger, re.Pattern) and trigger.search(content)
        str_matches = isinstance(trigger, str) and trigger in content
        if re_matches or str_matches:
            await message.channel.send(response)
