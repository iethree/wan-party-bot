from discord.ext import commands
import random
import re

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
    if arg is None:
        await ctx.send(str(random.randint(1, 100)))
        return

    error_message = "Something went wrong. It's probably Ryan's fault."
    instruction = ""

    rolls, result = [], 0
    try:
        nice_arg = nice_dice(arg)
        if not re.search("(" + DICE_RE + r"\s*)*", nice_arg):
            raise Exception("haha")
        rolls, result = do_the_thing(nice_arg)
    except Exception as e: #yolo
        await ctx.send(error_message)
        print(e)
        return

    await ctx.send("I rolled: " + " ".join(rolls) + ", result: " + str(result))

