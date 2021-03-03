from discord.ext import commands
from discord.ext.commands import MemberConverter
import random
from giphy import *
import re
from get_error_message import (
    get_error_message_for_fun_times_everyone_loves_error_messages,
)
import sqlite3
import socket
import functools

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


def gimme_db(cmd_fn):
    @functools.wraps(cmd_fn)
    def fn(*args, **kwargs):
        if "db" in kwargs:
            return cmd_fn(*args, **kwargs)

        conn = sqlite3.connect(DATABASE)
        ret = cmd_fn(*args, **kwargs, db=conn.cursor())
        conn.commit()
        conn.close()
        return ret

    return fn


@bot.command()
@gimme_db
async def bet(ctx, bet: int, guess: str, *, db):
    await ctx.send(random_gif("vegas"))

    message = ""
    user_id = ctx.message.author.id

    user = db.execute("SELECT * FROM wanbux WHERE id = ?", (user_id,)).fetchone()

    is_naughty = (
        db.execute("SELECT * FROM naughty_list where id = ?", (user_id,)).fetchone()
        is not None
    )

    if bet <= 0 and not is_naughty:
        await ctx.send("Cute")
        return

    if user is None:
        message += f"Welcome to the WAN Casino {ctx.message.author.mention}."
        message += " Have 5 Wanbux on the house.\n"
        update_balance(user_id, 5, db=db)

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
    db.execute("UPDATE wanbux SET balance = ? WHERE id = ?", (new_balance, user_id))

    await ctx.send(message)


@gimme_db
async def get_balance(user_id, *, db):
    if row := db.execute(
        "SELECT balance FROM wanbux WHERE id = ?", (user_id,)
    ).fetchone():
        return row[0]
    return update_balance(user_id, 0)


@gimme_db
async def update_balance(user_id, update, *, db):
    return db.execute(
        "INSERT INTO wanbux (id, balance) VALUES (?, ?)"
        " ON DUPLICATE KEY UPDATE balance = ?",
        (user_id, update, update),
    )


@bot.command(name="balance")
async def eval_balance(ctx):
    balance = get_balance(ctx.message.author.id)
    await ctx.send(f"{ctx.message.author.mention}'s balance is {balance} wanbux")


@bot.command
@gimme_db
async def beg(ctx, *, db):
    author = ctx.message.author

    balance = get_balance(author.id, db=db)
    if balance == 0:
        await update_balance(author.id, 1, db=db)
        await ctx.send(f"Try not to spend it all in one place {author.mention} 😎")
    elif row is not None and row[0] > 0:
        await ctx.send("🖕")
    else:
        await ctx.send(get_error_message_for_fun_times_everyone_loves_error_messages())


# @bot.command
# async def pay(ctx, member: MemberConverter):
#     # etc you get it


@bot.command()
@gimme_db
async def rob(ctx, victim: MemberConverter, *, db):
    thief = ctx.message.author
    stolen = get_balance(victim.id, db=db)
    new_balance = get_balance(thief.id, db=db) + stolen
    update_balance(thief.id, new_balance)
    update_balance(victim.id, 0)


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
async def sql(ctx, *, arg=None):
    if "drop table" in arg.lower():
        result = "🖕"
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
