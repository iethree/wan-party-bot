import discord
import os
import openai
from datetime import datetime, timedelta

openai.api_key = os.getenv("OPENAI_API_KEY")

async def summarize_channel(channel):
    """
    Summarizes the conversations in a channel from the last 48 hours.
    """
    messages = []
    forty_eight_hours_ago = datetime.now(datetime.UTC) - timedelta(hours=48)

    async for message in channel.history(limit=None, after=forty_eight_hours_ago):
        messages.append(f"{message.author.display_name}: {message.content}")

    if not messages:
        return "No messages in the last 48 hours to summarize."

    prompt = "Summarize the following conversation:\n\n" + "\n".join(messages)

    try:
        response = openai.ChatCompletion.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "system", "content": "You are a helpful assistant that summarizes conversations."},
                {"role": "user", "content": prompt},
            ],
        )
        return response.choices[0].message.content
    except Exception as e:
        return f"Error summarizing conversation: {e}"