import random
from blacklist import is_blacklisted_channel
from openai import OpenAI
from datetime import date
from client import client
ai_client = OpenAI()

gpt_model = "gpt-5-nano"\

def get_personality():
    standard_personality = "Your name is WanBot, aka <@" + client.user +">, and you are a helpful robot in a discord server with a keen sense of humor that does not inhibit your helpfulness "
    conditional_personality = get_conditional_prompts()
    return standard_personality + conditional_personality


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

limit_context = {"role": "system", "content": "responses absolutely cannot exceed 1800 characters" }

context_buffer_size = 10
context_buffer = []

def add_to_context(role, msg):
    """
    Add a message to the context buffer, maintaining a maximum size.
    If the buffer exceeds the size limit, remove the oldest message.
    """
    context_buffer.append({
        "role": role,
        "content": msg
    })
    if len(context_buffer) > context_buffer_size:
        context_buffer.pop(0)

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
            {"role": "system", "content": get_personality()},
            {"role": "user", "content": "write a short comeback to " + msg }
        ]
    )
    print(completion.choices[0].message.content)
    return completion.choices[0].message.content

def get_tldr_response(msg):
    completion = ai_client.chat.completions.create(
        model=gpt_model,
        messages=[
            {"role": "system", "content": get_personality()},
            {"role": "user", "content": "write an extremely short and mildly flippant tldr summary of: " + msg }
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

async def get_bot_response(message):
    msg = message.content

    messages = [
        {"role": "system", "content": get_personality()},
        limit_context,
        *context_buffer
    ]

    messages.append({"role": "user", "content": msg})

    quoted_msg = await get_quoted_msg(message)
    quoted_context = {"role": "user", "content": "the previous message is responding to: " + quoted_msg.content} if quoted_msg is not None else None

    if quoted_context is not None:
        messages.append(quoted_context)

    completion = ai_client.chat.completions.create(
        model=gpt_model,
        messages=messages
    )
    response = completion.choices[0].message.content
    add_to_context("user", msg)
    add_to_context("assistant", response)

    print(response)
    return response

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

    quoted_msg = await get_quoted_msg(message)

    if quoted_msg is None:
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

    quoted_msg = await get_quoted_msg(message)

    if quoted_msg is None:
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

    quoted_msg = await get_quoted_msg(message)

    if quoted_msg is None:
        await message.add_reaction("🤷")
        return

    try:
        msg = get_person_response(personality, quoted_msg.content)
    except Exception as e:
        print("error getting ai comeback")
        print(e)
        await message.add_reaction("🫣")

    await quoted_msg.reply(msg)

async def tldr(message):
    try:
        if is_blacklisted_channel(message.channel.name):
            await message.add_reaction("🙅‍♀️")
            return
    except Exception as e:
        print("error checking blacklist")

    try:
        quoted_msg = await get_quoted_msg(message)
    except Exception as e:
        await message.add_reaction("🤷")
        return

    try:
        await think(message)
        msg = get_tldr_response(quoted_msg.content)
    except Exception as e:
        print("error getting ai tldr response")
        print(e)
        msg = get_comeback(quoted_msg.content)

    await quoted_msg.reply(msg)
    await unthink(message)

async def bot_response(message):
    print('responding to ' + message.content)
    try:
        if is_blacklisted_channel(message.channel.name):
            await message.add_reaction("🙅‍♀️")
            return
    except Exception as e:
        print("error checking blacklist")

    try:
        await think(message)
        msg = await get_bot_response(message)
        await unthink(message)
    except Exception as e:
        print("error getting ai bot response")
        print(e)
        await message.add_reaction("🤷‍♀️")
        return

    await message.reply(msg)

async def get_quoted_msg(message):
    try:
        quoted_msg = await message.channel.fetch_message(message.reference.message_id)
        return quoted_msg
    except Exception as e:
        print('error getting quoted message')
        return None


async def think(msg):
    await msg.add_reaction("🤔")

async def unthink(msg):
    await msg.remove_reaction("🤔", client.user)
