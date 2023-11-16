import sqlite3

DATABASE = "wanparty.db"

blacklist = ["sigh-politics", "bible", "anglicanism", "formative movie crushes of the youthful era"]

async def quote(message):
    if message.channel.name.lower() in blacklist:
        await message.add_reaction("🙅‍♀️")
        return

    try:
        quoted_msg = await message.channel.fetch_message(message.reference.message_id)
    except Exception as e:
        await message.channel.send("you probably didn't quote something, or the dev was too lazy to handle the error right")
        return

    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()

    try:
        print('quoting ' + quoted_msg.content)
        cursor.execute(
            "INSERT INTO quotes(user_id, quote) VALUES(?,?)",
            (quoted_msg.author.id, quoted_msg.content),
        )
        conn.commit()
        conn.close()
    except Exception as e:
        await message.channel.send(e)
        await message.add_reaction("❌")
        return

    await message.add_reaction("✅")
