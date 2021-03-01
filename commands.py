from discord.ext import commands
import discord

bot = commands.Bot(command_prefix="/")


@commands.command()
async def puppet(ctx, channel_name, msg):
    channels = bot.get_all_channels()
    for channel in channels:
        if channel.name == channel_name:
            await channel.send(msg)

bot.add_command(puppet)