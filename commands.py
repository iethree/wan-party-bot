from discord.ext import commands
from discord.ext.commands import MemberConverter
import random
import re
from get_error_message import (
    get_error_message_for_fun_times_everyone_loves_error_messages,
)
import sqlite3
import socket

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

    is_naughty = bet_cursor.execute(
        "SELECT * FROM naughty_list where id = ?",
        (user_id,)
    ).fetchone() is not None

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
    bet_cursor.execute("UPDATE wanbux SET balance = ?", (new_balance,))

    conn.commit()
    conn.close()
    await ctx.send(message)


@bot.command()
async def balance(ctx):
    conn = sqlite3.connect(DATABASE)
    bal_cursor = conn.cursor()
    row = bal_cursor.execute(
        "SELECT balance FROM wanbux WHERE id = ?", (ctx.message.author.id,)
    ).fetchone()
    if row is not None:
        await ctx.send(f"{ctx.message.author.mention}'s balance is {row[0]} wanbux")
        return

    await ctx.send(f"{ctx.message.author.mention} doesn't have a balance")


# @bot.command
# async def pay(ctx, member: MemberConverter):
#     # etc you get it

# @bot.command
# async def beg(ctx):
# get 5 dollars from wanbot if balance = 0


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
