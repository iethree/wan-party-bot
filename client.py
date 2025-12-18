import discord

intents = discord.Intents.default()
intents.messages = True
intents.message_content = True
intents.reactions = True
# intents.polls = True

client = discord.Client(intents=intents)

