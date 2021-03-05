from discord.ext import commands
from discord.ext.commands import MemberConverter
import random
import re
from get_error_message import get_error_message_for_fun_times_everyone_loves_error_messages
import sqlite3
import socket

DATABASE = '/tmp/wanparty.db' if socket.gethostname() == "wan-party-bot" else "wanparty.db"

bot = commands.Bot(command_prefix="/")

INSTRUCTIONS = {
    "+": (lambda x, y: x + y),
    "-": (lambda x, y: x - y),
    "*": (lambda x, y: x * y),
    "/": (lambda x, y: x / y),
    "^": (lambda x, y: x ** y),
    "‽": (lambda x, y: 42)
}

INS_RE = "[" + re.escape("".join(INSTRUCTIONS.keys())) + "]"
DICE_RE = r"(\d+)\s*d\s*(\d+)\s*(" + INS_RE + r"\s*\d+)?"

@bot.command()
async def bet(ctx, bet: int, guess: str):
    if bet <= 0:
        await ctx.send("Cute")
        return
    
    message = ""
    user_id = ctx.message.author.id
    conn = sqlite3.connect(DATABASE)
    bet_cursor = conn.cursor()
    
    user = bet_cursor.execute("SELECT * FROM wanbux WHERE id = ?" , (user_id,)).fetchone()
    
    if user is None:
        message += f"Welcome to the WAN Casino {ctx.message.author.mention}. Have 5 Wanbux on the house.\n "
        bet_cursor.execute("INSERT INTO wanbux(id, balance) VALUES(?, ?)", (user_id, 5,))
    
    balance = user[1] if user is not None else 5
    
    if (bet > balance):
        await ctx.send(f"Your bet is too high. I'm going to assume you're betting "
                       f"everything you have, which is {balance} wanbux.\n")
        bet = balance
    
    flip = random.choice(["heads", "tails"])
    message += f"I flipped {flip}. "
    
    if flip == guess.strip().lower():
        message += f"You won {bet}! You have {balance + bet} wanbux now."
        bet_cursor.execute("UPDATE wanbux SET balance = ?", (balance + bet,))
    else:
        message += f"You lose {bet} wanbux! You have {balance - bet} total now. "
        message += "You're broke now! Get lost, ya bum." if balance - bet == 0 else ""
        bet_cursor.execute("UPDATE wanbux SET balance = ?", (balance - bet,))
        
    conn.commit()
    conn.close()
    await ctx.send(message)

@bot.command()
async def balance(ctx):
    conn = sqlite3.connect(DATABASE)
    bal_cursor = conn.cursor()
    row = bal_cursor.execute("SELECT balance FROM wanbux WHERE id = ?",
                             (ctx.message.author.id,)).fetchone()
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
    except Exception as e: #yolo
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
    from markov_haiku_discord import gen_haiku
    # identify which user to search for

    # pull users comments

    # train haiku_bot on comments

    # train haiku_bot on comments & generate a haiku
    gen_haiku("COMMENTS_STRING")
    await ctx.send("Sorry, hauiku_bot is new at this and currently under construction. Please be patient.")
    return