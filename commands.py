from discord.ext import commands
from discord.ext.commands import MemberConverter
from datetime import *
import random
from giphy import *
import re
from get_error_message import (
    get_error_message_for_fun_times_everyone_loves_error_messages,
)
import sqlite3
import socket
from markov_haiku_discord import gen_haiku

DATABASE = "wanparty.db"

bot = commands.Bot(command_prefix="/")

INSTRUCTIONS = {
    "+": (lambda x, y: x + y),
    "-": (lambda x, y: x - y),
    "*": (lambda x, y: x * y),
    "/": (lambda x, y: x / y),
    "^": (lambda x, y: x ** y),
    "‽": (lambda x, y: 42),
}

INS_RE = "[" + re.escape("".join(INSTRUCTIONS.keys())) + "]"
DICE_RE = r"(\d+)\s*d\s*(\d+)\s*(" + INS_RE + r"\s*\d+)?"


@bot.command()
async def bet(ctx, bet: int, guess: str):
    message = ""
    user_id = ctx.message.author.id
    conn = sqlite3.connect(DATABASE)
    bet_cursor = conn.cursor()

    user = bet_cursor.execute(
        "SELECT * FROM wanbux WHERE id = ?", (user_id,)
    ).fetchone()

    is_naughty = (
        bet_cursor.execute(
            "SELECT * FROM naughty_list where id = ?", (user_id,)
        ).fetchone()
        is not None
    )

    if bet <= 0 and not is_naughty:
        await ctx.send("Cute")
        return

    if user is None:

        message += f"Welcome to the WAN Casino {ctx.message.author.mention}."
        message += " Have 5 Wanbux on the house.\n"
        bet_cursor.execute(
            "INSERT INTO wanbux(id, balance) VALUES(?, ?)",
            (
                user_id,
                5,
            ),
        )

    balance = user[1] if user is not None else 5

    if bet > balance:
        if not is_naughty:
            await ctx.send(
                f"Your bet is too high. I'm going to assume you're betting "
                f"everything you have, which is {balance} wanbux.\n"
            )
            bet = balance

    flip = random.choice(["heads", "tails"])
    message += f"I flipped {flip}."

    is_win = flip == guess.strip().lower()
    new_balance = balance + bet if is_win else balance - bet
    message += f" You {'won' if is_win else 'lost'} {bet} wanbux!"
    message += f" You have {new_balance} wanbux now."
    if new_balance == 0:
        message += " You're broke now! Get lost, ya bum."
    bet_cursor.execute(
        "UPDATE wanbux SET balance = ? WHERE id = ?", (new_balance, user_id)
    )

    conn.commit()
    conn.close()
    await ctx.send(message)


# fuck around and find out
@bot.command(name="yeet")
async def yeet_bet(ctx, guess: str):
    balance = await get_balance(ctx.message.author.id)
    await ctx.send("https://gph.is/g/4wMRo3n")
    bet_amount = random.randrange(1, balance[0])
    await bet(ctx, bet_amount, guess)


async def get_balance(user_id):
    conn = sqlite3.connect(DATABASE)
    bal_cursor = conn.cursor()
    row = bal_cursor.execute(
        "SELECT balance FROM wanbux WHERE id = ?", (user_id,)
    ).fetchone()
    return row


async def update_balance(user_id, update):
    conn = sqlite3.connect(DATABASE)
    bal_cursor = conn.cursor()
    updated_row = bal_cursor.execute(
        "UPDATE wanbux set balance = ? WHERE id = ?", (update, user_id)
    )
    conn.commit()
    conn.close()
    return updated_row


# set balance command for testing purposes
@bot.command(name="devset")
async def dev_set(ctx, amount: int):
    await update_balance(ctx.message.author.id, amount)
    balance = await get_balance(ctx.message.author.id)
    await ctx.send(f"Your balance has been set to {balance[0]}")


@bot.command(name="balance")
async def eval_balance(ctx):
    balance = await get_balance(ctx.message.author.id)
    if balance is not None:
        await ctx.send(f"{ctx.message.author.mention}'s balance is {balance[0]} wanbux")
        return

    await ctx.send(f"{ctx.message.author.mention} doesn't have a balance")


@bot.command(name="myid")
async def my_id(ctx):
    user_id = ctx.message.author.id
    user_name = ctx.message.author.display_name
    await ctx.send(ctx.message.author.id)


@bot.command(name="id")
async def id(ctx):
    if len(ctx.message.raw_mentions):
        await ctx.send(
            "> " + "\n > ".join(map(lambda m: str(m), ctx.message.raw_mentions))
        )


@bot.command()
async def beg(ctx):
    row = await get_balance(ctx.message.author.id)
    if row is not None and row[0] == 0:
        await update_balance(ctx.message.author.id, 1)
        await ctx.send(
            f"Try not to spend it all in one place {ctx.message.author.mention} 😎"
        )
    elif row is not None and row[0] > 0:
        await ctx.send("🖕")
    else:
        await ctx.send(get_error_message_for_fun_times_everyone_loves_error_messages())


@bot.command()
async def rob(ctx):
    [balance] = await get_balance(ctx.message.author.id)
    for victim in ctx.message.mentions:
        [stolen] = await get_balance(victim.id)
        balance += stolen
        await update_balance(victim.id, 0)
    await update_balance(ctx.message.author.id, balance)


# @bot.command
# async def pay(ctx, member: MemberConverter):
#     # etc you get it


@bot.command()
async def rollin(ctx):
    await ctx.send("Aww yeah 😎")


@bot.command()
async def puppet(ctx, channel_name, msg):
    channels = bot.get_all_channels()
    for channel in channels:
        if channel.name == channel_name:
            await channel.send(msg)


def nice_dice(dice):
    return dice.strip().lower()


def do_the_thing(dice):
    rolls = []
    result = 0
    for (dice_count, face_count, math) in re.findall(DICE_RE, dice):
        sub_result = 0
        for _ in range(int(dice_count)):
            roll = random.randint(1, int(face_count))
            rolls.append(str(roll))
            sub_result += roll
        if math != "":
            sub_result = INSTRUCTIONS[math[0]](sub_result, int(math[1:]))
        result += sub_result
    return rolls, result


@bot.command()
async def roll(ctx, *, arg=None):
    """XdY+Z AdB+C etc"""
    if arg is None:
        await ctx.send(str(random.randint(1, 100)))
        return

    instruction = ""

    rolls, result = [], 0
    try:
        nice_arg = nice_dice(arg)
        if not re.search("(" + DICE_RE + r"\s*)*", nice_arg):
            raise Exception("haha")
        rolls, result = do_the_thing(nice_arg)
    except Exception as e:  # yolo
        await ctx.send(get_error_message_for_fun_times_everyone_loves_error_messages())
        print(e)
        return

    await ctx.send("I rolled: " + " ".join(rolls) + ", result: " + str(result))


@bot.command()
async def haiku(ctx, *, arg=None):
    """
    Call as /haiku @targetuser.
    Will generate a markov chain haiku using the
    channel's history from that user's comments.
    """

    # trust no one
    if len(ctx.message.raw_mentions) < 1:
        await ctx.send("I need a muse. @somebody, fool.")
        return

    try:
        # identify which user to search for
        this_guild = ctx.message.guild
        this_channel = ctx.message.channel
        target_user = ctx.message.raw_mentions[0]

        # pull users comments
        haiku_list = []
        async for m in ctx.message.channel.history(limit=1500):
            if m.author.id == target_user:
                content = m.content.split(" ")
                if len(content) > 4:
                    haiku_list.append(" ".join(content[:]))
        haiku_string = " ".join(haiku_list)

        # train haiku_bot on comments & generate a haiku
        result_list = gen_haiku(haiku_string)

        result_string = "\n> ".join(map(lambda line: " ".join(line), result_list))
        await ctx.send("> " + result_string)
    except Exception as e:
        await ctx.send("Inspiration eludes me, or " + repr(e) + " one of the two...")

    return


@bot.command()
async def rhyme(ctx, *, arg=None):
    """
    Call as /rhyme 'word'.
    Will generate a list of rhymes from the word.
    """

    # trust no one
    word = ctx.message.content
    if word[1:].split(" ") != 1:
        await ctx.send("I need a word to contemplate. `/rhyme word`, fool.")
        return

    try:
        from corpora.cmudict.rhymes import rhyme

        rhymes = rhyme(word)
        output = ""
        for pronunciation in rhymes.keys():
            output += (
                pronunciation,
                " rhymes with:\n",
                rhymes[pronunciation],
                f"\n\nA total of {len(rhymes[pronunciation])} rhymes.",
            )
        await ctx.send("> " + output)
    except Exception as e:
        await ctx.send(
            "I'm having trouble with this one, you're probably making it up, or "
            + repr(e)
            + ", one of the two..."
        )

    return


@bot.command()
async def sql(ctx, *, arg=None):

    if is_naughty(ctx.message.author.id):
        result = "Your SQL privileges have been revoked while in jail"
    else:
        try:
            conn = sqlite3.connect(DATABASE)
            cursor = conn.cursor()
            cursor.execute(arg)
            result = cursor.fetchall()
            result = "\n".join(map(str, result))
            conn.commit()
            conn.close()
        except Exception as e:
            result = "Error: " + str(e)

    await ctx.send("```sql\n" + str(result) + "\n```")


@bot.command()
async def jail(ctx, action=None, person=None):

    if action == "break" or action == "bust":
        if is_naughty(ctx.message.author.id):
            await ctx.send("You can't break anyone out from the inside")
        elif person == None:
            await ctx.send("You gotta @someone to bust out")
        elif len(ctx.message.raw_mentions):
            for jailbird in ctx.message.raw_mentions:
                if is_naughty(jailbird):
                    bust_out(jailbird)
                    await ctx.send(f"busted out <@!{jailbird}> !")
                else:
                    await ctx.send(f"<@!{jailbird}> isn't in jail!")
    if action == "bribe":
        balance = await get_balance(ctx.message.author.id)
        amount = random.randrange(1, balance[0])
        await update_balance(ctx.message.author.id, amount)
        bust_out(ctx.message.author.id)
        await ctx.send(f"{ctx.message.author.mention} has been shown mercy")

    if (action == "beg" or action == "mercy") and is_naughty(ctx.message.author.id):
        beg_mercy(ctx.message.author.id)
        await ctx.send(f"{ctx.message.author.mention} has been shown mercy")
    if action == "frame" and person != None:
        for victim in ctx.message.raw_mentions:
            if not is_naughty(victim):
                frame(victim)
                await ctx.send(f"framed <@!{victim}> !")

    await jail_update(ctx)


async def jail_update(ctx):
    conn = sqlite3.connect(DATABASE)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    presumed_guilty = 100
    sentence = "+1 hour"

    cursor.execute(
        "SELECT jail.id, users.name FROM jail LEFT JOIN users ON jail.id = users.id WHERE jail.out_at < datetime();"
    )
    getting_out = cursor.fetchall()

    cursor.execute(
        "SELECT wanbux.id, users.name, balance FROM wanbux LEFT JOIN users ON users.id = wanbux.id WHERE wanbux.balance > ?;",
        (presumed_guilty,),
    )
    going_in = cursor.fetchall()

    # let em out when they've done their time
    cursor.execute("DELETE FROM jail WHERE out_at < datetime();")

    # put em in when they did wrong and confiscate their money
    cursor.execute(
        "INSERT INTO JAIL (id, out_at) SELECT id, datetime('now', ?) FROM wanbux WHERE wanbux.balance > ?;",
        (
            sentence,
            presumed_guilty,
        ),
    )
    cursor.execute(
        "UPDATE wanbux SET balance = 0 WHERE balance > ?;", (presumed_guilty,)
    )

    cursor.execute(
        "SELECT jail.id, users.name, jail.out_at FROM jail LEFT JOIN users ON jail.id = users.id WHERE jail.out_at > datetime();"
    )
    staying_in = cursor.fetchall()

    conn.commit()
    conn.close()

    response = ""

    if len(getting_out):
        response += "\n**They've done their time, they're getting out:**\n"
        response += (
            "```python\n"
            + "\n".join(map(lambda u: u["name"] or str(u["id"]), getting_out))
            + "\n```"
        )
    if len(going_in):
        response += "\n**Caught red-handed, they're going in:**\n"
        response += (
            "```python\n"
            + "\n".join(
                map(
                    lambda u: (u["name"] or str(u["id"]))
                    + ": "
                    + str(u["balance"])
                    + " wbx seized",
                    going_in,
                )
            )
            + "\n```"
        )
    if len(staying_in):
        response += "\n**Jailhouse Census:**\n"
        response += (
            "```sql\n"
            + "\n".join(
                map(
                    lambda u: (u["name"] or str(u["id"]))
                    + ": releasing in "
                    + time_until(u["out_at"]),
                    staying_in,
                )
            )
            + "\n```"
        )

    await ctx.send(response or "Jail's empty!")


# doesn't work
@bot.command()
async def who(ctx):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()

    try:
        members = ctx.message.guild.members
        print("channel " + str(ctx.message.guild))
        print("members " + str(members))
        for m in members:
            print(str(m.id) + " : " + m.display_name)
            cursor.execute(
                "INSERT INTO users(id, name) VALUES(?, ?) ON CONFLICT(id) DO UPDATE SET name=?",
                (
                    m.id,
                    m.display_name,
                    m.display_name,
                ),
            )
    except Exception as e:
        print(repr(e))

    conn.commit()
    conn.close()
    await ctx.message.add_reaction("✅")


def time_until(time):
    time = datetime.strptime(time, "%Y-%m-%d %H:%M:%S")
    delta = time - datetime.now()
    return str(delta)


def is_naughty(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    is_naughty = (
        cursor.execute("SELECT * FROM jail where id = ?", (user_id,)).fetchone()
        is not None
    )
    conn.commit()
    conn.close()
    return is_naughty


def bust_out(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    cursor.execute("DELETE FROM jail WHERE id = ?", (user_id,))
    conn.commit()
    conn.close()


def frame(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    cursor.execute("UPDATE wanbux SET balance = 99999 WHERE id = ?", (user_id,))
    conn.commit()
    conn.close()


def beg_mercy(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    cursor.execute(
        "UPDATE jail SET out_at=datetime(out_at, '-1 minute') WHERE id = ?", (user_id,)
    )
    conn.commit()
    conn.close()
