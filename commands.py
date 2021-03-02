from discord.ext import commands
import random
import re
import sys

bot = commands.Bot(command_prefix="/")


@bot.command()
async def puppet(ctx, channel_name, msg):
    channels = bot.get_all_channels()
    for channel in channels:
        if channel.name == channel_name:
            await channel.send(msg)

def nice_dice(dice):
    return dice.strip().lower()

@bot.command()
async def roll(ctx, *, arg=None):
    if arg is None:
        await ctx.send(str(random.randint(1, 100)))
        return
    dice = arg.strip().lower()
    dice_count = 1
    result = 0
    error_message = "Something went wrong. It's probably Ryan's fault."
    instruction = ""
    rolls = []

    dice_list = re.split("([d\+\*\-])", dice)

    try:
        for i, x in enumerate(dice_list):
            el = x.strip()
            if not el.isdigit():
                instruction = el
                continue

            if instruction == "":
                dice_count = int(el)
                continue

            if instruction == "d":
                for x in range(dice_count):
                    roll = random.randint(1, int(el))
                    rolls.append(str(roll))
                    result += roll

            if instruction == "+":
                result += int(el)

            if instruction == "-":
                result -= int(el)

            if instruction == "*":
                result *= int(el)


    except Exception as e: #yolo
        await ctx.send(error_message)
        print(e)
        return

    await ctx.send("I rolled: " + " ".join(rolls) + ", result: " + str(result))

