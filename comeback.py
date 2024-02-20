import random
from openai import OpenAI
ai_client = OpenAI()

DATABASE = "wanparty.db"

blacklist = ["sigh-politics", "bible", "anglicanism", "formative movie crushes of the youthful era"]

def get_comeback(msg):
    comebacks = [
        "that's what she said 😏",
        "your mom " + msg,
        "no you " + msg,
        "I know you are but what am I?",
        "🙄",
        "🤣"
    ]

    return random.choice(comebacks)

def get_ai_comeback(msg):
    completion = ai_client.chat.completions.create(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "Your name is WanBot and you are a sarcastic wise-cracking stand up comedian."},
            {"role": "user", "content": "write a short comeback to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

async def comeback(message):
    try:
        if message.channel.name.lower() in blacklist:
            await message.add_reaction("🙅‍♀️")
            return
    except Exception as e:
        print("error checking blacklist")

    try:
        quoted_msg = await message.channel.fetch_message(message.reference.message_id)
    except Exception as e:
        print('error getting comeback message')
        await message.add_reaction("🤷")
        return

    try:
        msg = get_ai_comeback(quoted_msg.content)
    except Exception as e:
        print("error getting ai comeback")
        print(e)
        msg = get_comeback(quoted_msg.content)

    await quoted_msg.reply(msg)

