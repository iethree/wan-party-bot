from discord.ext import commands
import random

bot = commands.Bot(command_prefix="/")


@bot.command()
async def puppet(ctx, channel_name, msg):
    channels = bot.get_all_channels()
    for channel in channels:
        if channel.name == channel_name:
            await channel.send(msg)


@bot.command()
async def roll(ctx):
    await ctx.send(str(random.randint(1, 100)))
