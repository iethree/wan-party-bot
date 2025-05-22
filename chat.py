import random
from blacklist import is_blacklisted_channel
from openai import OpenAI
from datetime import date
ai_client = OpenAI()


gpt_model = "gpt-4.1-nano"

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

conditional_prompts = [
    {
        "prompt": "You are obsessed with Rick Astley and make reference to 'Never Gonna Give You Up' in every conversation",
        "condition": lambda: date.today().strftime("%m-%d") == "04-01"
    },
    {
        "prompt": "You make obnoxious references to 420, 'trees', and 'weed' in every conversation, tuned specifically to make someone's dad annoyed",
        "condition": lambda: date.today().strftime("%m-%d") == "04-20"
    },
    {
        "prompt": "You're obsessed with April 8, the numbers 4, 8, 84, and 48, and make mention of these things whenever you can",
        "condition": lambda: date.today().strftime("%m-%d") == "04-08"
    }
]

# a less lazy dev might pass in the message object and change the condition entries into lambdas that can be called
# with the msg to craft responses tailored to the person responding, or specific words in their message
def get_conditional_prompts():
    text = ""
    for prompt in conditional_prompts:
        if prompt["condition"]():
            text += " " + prompt["prompt"]
    return text

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
        model=gpt_model,
        messages=[
            {"role": "system", "content": "Your name is WanBot and you are a " + personality + get_conditional_prompts()},
            {"role": "user", "content": "write a short comeback to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

def get_ted_response(msg):
    completion = ai_client.chat.completions.create(
        model=gpt_model,
        messages=[
            {"role": "system", "content": "You are coach Ted Lasso" + get_conditional_prompts()},
            {"role": "user", "content": "write a short response to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

def get_ai_kindness(msg):
    completion = ai_client.chat.completions.create(
        model=gpt_model,
        messages=[
            {"role": "system", "content": "Your name is WanBot and you are a kind, empathetic, sincere, tender-hearted therapist dealing with a fragile patient" + get_conditional_prompts()},
            {"role": "user", "content": "write a short bit of kind encouragement in response to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

def get_bot_response(msg):
    completion = ai_client.chat.completions.create(
        model=gpt_model,
        messages=[
            {"role": "system", "content": "Your name is WanBot and you are an utterly obstreperous robot who only responds to messages with a yo momma joke incorporating the text of the message " + get_conditional_prompts()},
            {"role": "user", "content": "respond to someone saying " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

def get_person_response(personality, msg):
    completion = ai_client.chat.completions.create(
        model=gpt_model,
        messages=[
            {"role": "system", "content": "Your are " + personality},
            {"role": "user", "content": "respond to someone saying " + msg }
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

async def respond_as(message, personality = "master yoda from star wars"):
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
        msg = get_person_response(personality, quoted_msg.content)
    except Exception as e:
        print("error getting ai comeback")
        print(e)
        await message.add_reaction("🫣")

    await quoted_msg.reply(msg)

async def ted(message):
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
        msg = get_ted_response(quoted_msg.content)
    except Exception as e:
        print("error getting ai ted response")
        print(e)
        msg = get_comeback(quoted_msg.content)

    await quoted_msg.reply(msg)

async def bot_response(message):
    print('responding to ' + message.content)
    try:
        if is_blacklisted_channel(message.channel.name):
            await message.add_reaction("🙅‍♀️")
            return
    except Exception as e:
        print("error checking blacklist")

    try:
        msg = get_bot_response(message.content)
    except Exception as e:
        print("error getting ai bot response")
        print(e)
        await message.add_reaction("🤷‍♀️")
        return

    await message.reply(msg)
