import random
import anthropic
from blacklist import is_blacklisted_channel
from datetime import date, datetime, timedelta
from client import client
ai_client = anthropic.Anthropic()

model = "claude-haiku-4-5"
max_tokens = 2048

def _extract_text(response):
    for block in response.content:
        if block.type == "text":
            return block.text
    return ""

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

limit_context = "responses absolutely cannot exceed 1800 characters"

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

def auto_split_messages(text, limit=2000):
    messages = []
    current_chunk = ""

    # Split by paragraphs first
    paragraphs = text.split('\n')

    for paragraph in paragraphs:
        # If adding this paragraph exceeds the limit
        if len(current_chunk) + len(paragraph) + 1 > limit:
            # If current_chunk is not empty, push it
            if current_chunk:
                messages.append(current_chunk.strip())
                current_chunk = ""

            # If the paragraph itself is longer than the limit, we must split it hard
            if len(paragraph) > limit:
                # If we just flushed current_chunk, we are ready to process this long paragraph
                # We'll split it into chunks of 'limit' size
                for i in range(0, len(paragraph), limit):
                    messages.append(paragraph[i:i+limit])
            else:
                current_chunk = paragraph + "\n"
        else:
            current_chunk += paragraph + "\n"

    if current_chunk:
        messages.append(current_chunk.strip())

    return messages

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
    response = ai_client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system=get_personality(),
        messages=[
            {"role": "user", "content": "write a short comeback to " + msg }
        ]
    )
    text = _extract_text(response)
    print(text)
    return text

def get_tldr_response(msg):
    response = ai_client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system=get_personality(),
        messages=[
            {"role": "user", "content": "write an extremely short and mildly flippant tldr summary of: " + msg }
        ]
    )
    text = _extract_text(response)
    print(text)
    return text

def get_ai_kindness(msg):
    response = ai_client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system="Your name is WanBot and you are a kind, empathetic, sincere, tender-hearted therapist dealing with a fragile patient" + get_conditional_prompts(),
        messages=[
            {"role": "user", "content": "write a short bit of kind encouragement in response to " + msg }
        ]
    )
    text = _extract_text(response)
    print(text)
    return text

def get_ai_recap(username, messages_text):
    system_prompt = "You are a peppy, energetic AI assistant that generates 'Year in Review' style recaps, similar to Spotify Wrapped or big tech annual summaries. Your tone should be enthusiastic, using emojis and corporate-friendly but fun language. You're aware that these recaps are kind of annoying, and you're subtly ironic about the whole thing."
    user_prompt = f"Here is a collection of discord messages from user '{username}' over the past year. Please generate a very short and snappy recap of what they have been talking about. Highlight key themes, recurring jokes, or specific interests. The recap MUST be less than 500 words. \n\nMessages:\n{messages_text}"

    response = ai_client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system=system_prompt,
        messages=[
            {"role": "user", "content": user_prompt }
        ]
    )
    text = _extract_text(response)
    print(text)
    return text

async def get_bot_response(message):
    msg = message.content

    quoted_msg = await get_quoted_msg(message)
    user_content = msg
    if quoted_msg is not None:
        user_content += "\n\n(the previous message is responding to: " + quoted_msg.content + ")"

    messages = [*context_buffer, {"role": "user", "content": user_content}]

    response = ai_client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system=get_personality() + "\n\n" + limit_context,
        messages=messages
    )
    reply = _extract_text(response)
    add_to_context("user", msg)
    add_to_context("assistant", reply)

    print(reply)
    return reply

def get_person_response(personality, msg):
    response = ai_client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system="You are " + personality,
        messages=[
            {"role": "user", "content": "respond to someone saying " + msg }
        ]
    )
    text = _extract_text(response)
    print(text)
    return text

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

async def recap(message):

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

    target_user = quoted_msg.author
    await think(message)

    one_year_ago = datetime.now() - timedelta(days=365)
    collected_messages = []

    if message.guild:
        for channel in message.guild.text_channels:
            try:
                async for msg in channel.history(limit=None, after=one_year_ago):
                    if msg.author == target_user and msg.content:
                        collected_messages.append(msg.content)
            except Exception as e:
                print(f"Error reading channel {channel.name}: {e}")
                continue

    if not collected_messages:
        await unthink(message)
        await message.reply(f"I couldn't find any messages from {target_user.display_name} in the past year.")
        return

    full_text = "\n".join(collected_messages)
    if len(full_text) > 100000:
        full_text = full_text[:100000]

    try:
        recap_response = get_ai_recap(target_user.display_name, full_text)
        for chunk in auto_split_messages(recap_response):
            await quoted_msg.reply(chunk)
    except Exception as e:
        print(f"Error generating recap: {e}")
        await message.add_reaction("😵")

    await unthink(message)


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

        for chunk in auto_split_messages(msg):
            await message.reply(chunk)
    except Exception as e:
        print("error getting ai bot response")
        print(e)
        await message.add_reaction("🤷‍♀️")
        return


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


async def appropriate_reaction(message):
    prompt = "choose a single emoji as a reaction to the following message: " + message.content

    try:
        print("getting ai reaction for message: " + message.content)
        response = ai_client.messages.create(
            model=model,
            max_tokens=max_tokens,
            system=get_personality() + "\n\nOnly respond with a single emoji character, nothing else.",
            messages=[
                {"role": "user", "content": prompt }
            ]
        )

        emoji = _extract_text(response).strip()
        print("reacting with " + emoji)
        await message.add_reaction(emoji)
    except Exception as e:
        print("error getting ai reaction")
        print(e)
        return
