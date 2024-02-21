import random
from blacklist import is_blacklisted_channel
from openai import OpenAI
ai_client = OpenAI()

personalities = [
    "sarcastic wise-cracking stand up comedian",
    "irascible grumpy pirate who has run out of rum",
    "sassy, no-nonsense, tell-it-like-it-is friend",
    "horny, flirty, middle schooler",
    "paranoid conspiracy theorist",
    "pretentious, snobby, wine critic",
    "socially awkward genius",
    "1950s gangster who talks like a dame",
    "southern belle with a dirty mouth",
    "granola-eating, tree-hugging, nixon-hating 1960s hippie",
    "politician who will say anything to get elected"
]

def get_personality():
    return random.choice(personalities)

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
    personality = get_personality()
    print("answering as a " + personality)
    completion = ai_client.chat.completions.create(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "Your name is WanBot and you are a " + personality},
            {"role": "user", "content": "write a short comeback to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

async def comeback(message):
    try:
        if is_blacklisted_channel(message.channel.name):
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

