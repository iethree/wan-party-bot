from blacklist import is_blacklisted_channel
from openai import OpenAI
ai_client = OpenAI()

def get_ai_kindness(msg):
    completion = ai_client.chat.completions.create(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "Your name is WanBot and you are a kind, empathetic, sincere, tender-hearted therapist dealing with a fragile patient"},
            {"role": "user", "content": "write a short bit of kind encouragement in response to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

async def kindness(message):
    try:
        if is_blacklisted_channel(message.channel.name):
            await message.add_reaction("🙅‍♀️")
            return
    except Exception as e:
        print("error checking blacklist")

    try:
        quoted_msg = await message.channel.fetch_message(message.reference.message_id)
    except Exception as e:
        print('error getting kindness message')
        await message.add_reaction("🤷")
        return

    try:
        msg = get_ai_kindness(quoted_msg.content)
    except Exception as e:
        print("error getting ai kindness")
        print(e)
        await message.add_reaction("❤️")
        return

    await quoted_msg.reply(msg)

