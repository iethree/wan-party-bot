import random

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

    msg = get_comeback(quoted_msg.content)

    await quoted_msg.reply(msg)


