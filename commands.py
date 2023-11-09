import discord
from datetime import *
from dick import *
from sing import *
from giphy import *
from thinking import thinking
from leaderboards import get_leaderboards
import random
from client import client

import re
from get_error_message import (
    get_error_message_for_fun_times_everyone_loves_error_messages,
)
import sqlite3
from markov_haiku_discord import gen_haiku

DATABASE = "wanparty.db"

tree = discord.app_commands.CommandTree(client)

INSTRUCTIONS = {
    "+": (lambda x, y: x + y),
    "-": (lambda x, y: x - y),
    "*": (lambda x, y: x * y),
    "/": (lambda x, y: x / y),
    "^": (lambda x, y: x**y),
    "‽": (lambda x, y: 42),
}

INS_RE = "[" + re.escape("".join(INSTRUCTIONS.keys())) + "]"
DICE_RE = r"(\d+)\s*d\s*(\d+)\s*(" + INS_RE + r"\s*\d+)?"


@tree.command()
async def bet(ctx, bet: int, guess: str):
    message = ""
    user_id = interaction.message.author.id
    conn = sqlite3.connect(DATABASE)
    bet_cursor = conn.cursor()

    user = bet_cursor.execute(
        "SELECT * FROM wanbux WHERE id = ?", (user_id,)
    ).fetchone()

    is_naughty = (
        bet_cursor.execute(
            "SELECT * FROM naughty_list where id = ?", (user_id,)
        ).fetchone()
        is not None
    )

    if bet <= 0 and not is_naughty:
        await interaction.response.send_message("Cute")
        return

    if user is None:
        message += f"Welcome to the WAN Casino {interaction.message.author.mention}."
        message += " Have 5 Wanbux on the house.\n"
        bet_cursor.execute(
            "INSERT INTO wanbux(id, balance) VALUES(?, ?)",
            (
                user_id,
                5,
            ),
        )

    balance = user[1] if user is not None else 5

    if bet > balance:
        if not is_naughty:
            await interaction.response.send_message(
                f"Your bet is too high. I'm going to assume you're betting "
                f"everything you have, which is {balance} wanbux.\n"
            )
            bet = balance

    flip = random.choice(["heads", "tails"])
    message += f"I flipped {flip}."

    is_win = flip == guess.strip().lower()
    new_balance = balance + bet if is_win else balance - bet
    message += f" You {'won' if is_win else 'lost'} {bet} wanbux!"
    message += f" You have {new_balance} wanbux now."
    if new_balance == 0:
        message += " You're broke now! Get lost, ya bum."
    bet_cursor.execute(
        "UPDATE wanbux SET balance = ? WHERE id = ?", (new_balance, user_id)
    )

    conn.commit()
    conn.close()
    await interaction.response.send_message(message)


# fuck around and find out
@tree.command(name="yeet")
async def yeet_bet(ctx, guess: str):
    balance = await get_balance(interaction.message.author.id)
    await interaction.response.send_message("https://gph.is/g/4wMRo3n")
    bet_amount = random.randrange(1, balance[0])
    await bet(ctx, bet_amount, guess)


async def get_balance(user_id):
    conn = sqlite3.connect(DATABASE)
    bal_cursor = conn.cursor()
    row = bal_cursor.execute(
        "SELECT balance FROM wanbux WHERE id = ?", (user_id,)
    ).fetchone()
    return row


async def update_balance(user_id, update):
    conn = sqlite3.connect(DATABASE)
    bal_cursor = conn.cursor()
    updated_row = bal_cursor.execute(
        "UPDATE wanbux set balance = ? WHERE id = ?", (update, user_id)
    )
    conn.commit()
    conn.close()
    return updated_row


# set balance command for testing purposes
@tree.command(name="devset")
async def dev_set(ctx, amount: int):
    await update_balance(interaction.message.author.id, amount)
    balance = await get_balance(interaction.message.author.id)
    await interaction.response.send_message(f"Your balance has been set to {balance[0]}")


@tree.command(name="balance")
async def eval_balance(interaction):
    balance = await get_balance(interaction.message.author.id)
    if balance is not None:
        await interaction.response.send_message(f"{interaction.message.author.mention}'s balance is {balance[0]} wanbux")
        return

    await interaction.response.send_message(f"{interaction.message.author.mention} doesn't have a balance")


@tree.command(name="myid")
async def my_id(interaction):
    user_id = interaction.message.author.id
    user_name = interaction.message.author.display_name
    await interaction.response.send_message(interaction.message.author.id)


@tree.command(name="id")
async def id(interaction):
    if len(interaction.message.raw_mentions):
        await interaction.response.send_message(
            "> " + "\n > ".join(map(lambda m: str(m), interaction.message.raw_mentions))
        )


@tree.command()
async def beg(interaction):
    row = await get_balance(interaction.message.author.id)
    if row is not None and row[0] == 0:
        await update_balance(interaction.message.author.id, 1)
        await interaction.response.send_message(
            f"Try not to spend it all in one place {interaction.message.author.mention} 😎"
        )
    elif row is not None and row[0] > 0:
        await interaction.response.send_message("🖕")
    else:
        await interaction.response.send_message(get_error_message_for_fun_times_everyone_loves_error_messages())


@tree.command()
async def rob(interaction):
    [balance] = await get_balance(interaction.message.author.id)
    for victim in interaction.message.mentions:
        [stolen] = await get_balance(victim.id)
        balance += stolen
        await update_balance(victim.id, 0)
    await update_balance(interaction.message.author.id, balance)


# @tree.command
# async def pay(ctx, member: MemberConverter):
#     # etc you get it


@tree.command()
async def rollin(interaction):
    await interaction.response.send_message("Aww yeah 😎")


@tree.command()
async def puppet(ctx, channel_name: str, msg: str):
    channels = tree.get_all_channels()
    for channel in channels:
        if channel.name == channel_name:
            await channel.send(msg)


def nice_dice(dice):
    return dice.strip().lower()


def do_the_thing(dice):
    rolls = []
    result = 0
    for dice_count, face_count, math in re.findall(DICE_RE, dice):
        sub_result = 0
        for _ in range(int(dice_count)):
            roll = random.randint(1, int(face_count))
            rolls.append(str(roll))
            sub_result += roll
        if math != "":
            sub_result = INSTRUCTIONS[math[0]](sub_result, int(math[1:]))
        result += sub_result
    return rolls, result


@tree.command()
async def roll(ctx, *, arg: int = None):
    """XdY+Z AdB+C etc"""
    if arg is None:
        await interaction.response.send_message(str(random.randint(1, 100)))
        return

    instruction = ""

    rolls, result = [], 0
    try:
        nice_arg = nice_dice(arg)
        if not re.search("(" + DICE_RE + r"\s*)*", nice_arg):
            raise Exception("haha")
        rolls, result = do_the_thing(nice_arg)
    except Exception as e:  # yolo
        await interaction.response.send_message(get_error_message_for_fun_times_everyone_loves_error_messages())
        print(e)
        return

    await interaction.response.send_message("I rolled: " + " ".join(rolls) + ", result: " + str(result))


@tree.command()
async def haiku(ctx, *, arg: str = None):
    """
    Call as /haiku @targetuser.
    Will generate a markov chain haiku using the
    channel's history from that user's comments.
    """

    # trust no one
    if len(interaction.message.raw_mentions) < 1:
        await interaction.response.send_message("I need a muse. @somebody, fool.")
        return

    try:
        # identify which user to search for
        this_guild = interaction.message.guild
        this_channel = interaction.message.channel
        target_user = interaction.message.raw_mentions[0]

        # pull users comments
        haiku_list = []
        async for m in interaction.message.channel.history(limit=1500):
            if m.author.id == target_user:
                content = m.content.split(" ")
                if len(content) > 4:
                    haiku_list.append(" ".join(content[:]))
        haiku_string = " ".join(haiku_list)

        # train haiku_bot on comments & generate a haiku
        result_list = gen_haiku(haiku_string)

        result_string = "\n> ".join(map(lambda line: " ".join(line), result_list))
        await interaction.response.send_message("> " + result_string)
    except Exception as e:
        await interaction.response.send_message("Inspiration eludes me, or " + repr(e) + " one of the two...")

    return


@tree.command()
async def rhyme(ctx, *, arg: str = None):
    """
    Call as /rhyme 'word'.
    Will generate a list of rhymes from the word.
    """

    # trust no one
    word = arg  # interaction.message.content
    if len(word.split(" ")) != 1:
        await interaction.response.send_message("I need a word to contemplate. `/rhyme word`, fool.", word)
        return

    try:
        from rhymes import rhyme

        rhymes = rhyme(word)
        output = ""
        for pronunciation in rhymes.keys():
            if len(", ".join(rhymes[pronunciation])) > 1000:
                output += (
                    pronunciation
                    + " rhymes with:\n"
                    + ", ".join(rhymes[pronunciation][:30])
                    + "\n--snip--\n"
                    + ", ".join(
                        rhymes[pronunciation][len(rhymes[pronunciation]) - 30 :]
                    )
                    + f"\n\nA total of {len(rhymes[pronunciation])} rhymes."
                )
            else:
                output += (
                    pronunciation
                    + " rhymes with:\n"
                    + ", ".join(rhymes[pronunciation])
                    + f"\n\nA total of {len(rhymes[pronunciation])} rhymes."
                )
        await interaction.response.send_message("> " + output)
    except Exception as e:
        await interaction.response.send_message(
            "I'm having trouble with this one, you're probably making it up, or "
            + repr(e)
            + ", one of the two..."
        )

    return


@tree.command()
async def sql(ctx, *, arg: str = None):
    if is_naughty(interaction.message.author.id):
        result = "Your SQL privileges have been revoked while in jail"
    else:
        try:
            conn = sqlite3.connect(DATABASE)
            cursor = conn.cursor()
            cursor.execute(arg)
            result = cursor.fetchall()
            result = "\n".join(map(str, result))
            conn.commit()
            conn.close()
        except Exception as e:
            result = "Error: " + str(e)

    await interaction.response.send_message("```sql\n" + str(result) + "\n```")


@tree.command()
async def jail(ctx, action: str = None, person: str = None):
    if action == "break" or action == "bust":
        if is_naughty(interaction.message.author.id):
            await interaction.response.send_message("You can't break anyone out from the inside")
        elif person == None:
            await interaction.response.send_message("You gotta @someone to bust out")
        elif len(interaction.message.raw_mentions):
            for jailbird in interaction.message.raw_mentions:
                if is_naughty(jailbird):
                    bust_out(jailbird)
                    await interaction.response.send_message(f"busted out <@!{jailbird}> !")
                else:
                    await interaction.response.send_message(f"<@!{jailbird}> isn't in jail!")
    if action == "bribe":
        balance = await get_balance(interaction.message.author.id)
        amount = random.randrange(1, balance[0])
        await update_balance(interaction.message.author.id, amount)
        bust_out(interaction.message.author.id)
        await interaction.response.send_message(f"{interaction.message.author.mention} has been shown mercy")

    if (action == "beg" or action == "mercy") and is_naughty(interaction.message.author.id):
        beg_mercy(interaction.message.author.id)
        await interaction.response.send_message(f"{interaction.message.author.mention} has been shown mercy")
    if action == "frame" and person != None:
        for victim in interaction.message.raw_mentions:
            if not is_naughty(victim):
                frame(victim)
                await interaction.response.send_message(f"framed <@!{victim}> !")

    await jail_update(interaction)


async def jail_update(interaction):
    conn = sqlite3.connect(DATABASE)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    presumed_guilty = 100
    sentence = "+1 hour"

    cursor.execute(
        "SELECT jail.id, users.name FROM jail LEFT JOIN users ON jail.id = users.id WHERE jail.out_at < datetime();"
    )
    getting_out = cursor.fetchall()

    cursor.execute(
        "SELECT wanbux.id, users.name, balance FROM wanbux LEFT JOIN users ON users.id = wanbux.id WHERE wanbux.balance > ?;",
        (presumed_guilty,),
    )
    going_in = cursor.fetchall()

    # let em out when they've done their time
    cursor.execute("DELETE FROM jail WHERE out_at < datetime();")

    # put em in when they did wrong and confiscate their money
    cursor.execute(
        "INSERT INTO JAIL (id, out_at) SELECT id, datetime('now', ?) FROM wanbux WHERE wanbux.balance > ?;",
        (
            sentence,
            presumed_guilty,
        ),
    )
    cursor.execute(
        "UPDATE wanbux SET balance = 0 WHERE balance > ?;", (presumed_guilty,)
    )

    cursor.execute(
        "SELECT jail.id, users.name, jail.out_at FROM jail LEFT JOIN users ON jail.id = users.id WHERE jail.out_at > datetime();"
    )
    staying_in = cursor.fetchall()

    conn.commit()
    conn.close()

    response = ""

    if len(getting_out):
        response += "\n**They've done their time, they're getting out:**\n"
        response += (
            "```python\n"
            + "\n".join(map(lambda u: u["name"] or str(u["id"]), getting_out))
            + "\n```"
        )
    if len(going_in):
        response += "\n**Caught red-handed, they're going in:**\n"
        response += (
            "```python\n"
            + "\n".join(
                map(
                    lambda u: (u["name"] or str(u["id"]))
                    + ": "
                    + str(u["balance"])
                    + " wbx seized",
                    going_in,
                )
            )
            + "\n```"
        )
    if len(staying_in):
        response += "\n**Jailhouse Census:**\n"
        response += (
            "```sql\n"
            + "\n".join(
                map(
                    lambda u: (u["name"] or str(u["id"]))
                    + ": releasing in "
                    + time_until(u["out_at"]),
                    staying_in,
                )
            )
            + "\n```"
        )

    await interaction.response.send_message(response or "Jail's empty!")


# doesn't work
@tree.command()
async def who(interaction):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()

    try:
        members = interaction.message.guild.members
        print("channel " + str(interaction.message.guild))
        print("members " + str(members))
        for m in members:
            print(str(m.id) + " : " + m.display_name)
            cursor.execute(
                "INSERT INTO users(id, name) VALUES(?, ?) ON CONFLICT(id) DO UPDATE SET name=?",
                (
                    m.id,
                    m.display_name,
                    m.display_name,
                ),
            )
    except Exception as e:
        print(repr(e))

    conn.commit()
    conn.close()
    await interaction.message.add_reaction("✅")


def time_until(time):
    time = datetime.strptime(time, "%Y-%m-%d %H:%M:%S")
    delta = time - datetime.now()
    return str(delta)


def is_naughty(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    is_naughty = (
        cursor.execute("SELECT * FROM jail where id = ?", (user_id,)).fetchone()
        is not None
    )
    conn.commit()
    conn.close()
    return is_naughty


def bust_out(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    cursor.execute("DELETE FROM jail WHERE id = ?", (user_id,))
    conn.commit()
    conn.close()


def frame(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    cursor.execute("UPDATE wanbux SET balance = 99999 WHERE id = ?", (user_id,))
    conn.commit()
    conn.close()


def beg_mercy(user_id):
    conn = sqlite3.connect(DATABASE)
    cursor = conn.cursor()
    cursor.execute(
        "UPDATE jail SET out_at=datetime(out_at, '-1 minute') WHERE id = ?", (user_id,)
    )
    conn.commit()
    conn.close()


@tree.command()
async def game_poll(interaction):
    poll_text = open("./weekly_games_poll.txt", encoding="utf8").read()
    await interaction.response.send_message(poll_text)


@tree.command()
async def dick(interaction):
    dick = get_random_quote("dick").replace("\n", "\n> ")
    await interaction.response.send_message(f"> {dick} ")


@tree.command()
async def dickens(interaction):
    dickens = get_random_quote("dickens").replace("\n", "\n> ")
    await interaction.response.send_message(f"> {dickens} ")


@tree.command()
async def willy(interaction):
    willy = get_random_quote("willy").replace("\n", "\n> ")
    await interaction.response.send_message(f"> {willy} ")


@tree.command()
async def thomas(interaction):
    thomas = get_random_quote("thomas").replace("\n", "\n> ")
    await interaction.response.send_message(f"> {thomas} ")


@tree.command(name = "jane", description = "Get some wisdom from Jane Austen")
async def jane(interaction):
    jane = get_random_quote("jane").replace("\n", "\n> ")
    await interaction.response.send_message(f"> {jane} ")


@tree.command()
async def v(interaction):
    dwarf = get_dwarf_quote()
    await interaction.response.send_message(dwarf)


@tree.command()
async def rick(interaction):
    song = sing_to_me()
    await interaction.response.send_message(song)


@tree.command(description="Get a random quote")
async def sayquote(interaction):
    try:
        conn = sqlite3.connect(DATABASE)
        cursor = conn.cursor()
        q = cursor.execute("SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1").fetchone()
        conn.commit()
        conn.close()
    except Exception as e:
        await interaction.response.send_message(e)
        return

    await interaction.response.send_message(f"{q[1]} --<@{q[0]}>")


@tree.command()
async def quotestats(interaction):
    try:
        conn = sqlite3.connect(DATABASE)
        cursor = conn.cursor()
        q = cursor.execute(
            "SELECT user_id, COUNT(*) AS count FROM quotes GROUP BY user_id ORDER BY count DESC"
        ).fetchall()
        conn.commit()
        conn.close()
    except Exception as e:
        await interaction.response.send_message(e)
        return
    result = ""
    for row in q:
        result += f"<@{row[0]}> has been quoted {str(row[1])} times\n"
    await interaction.response.send_message(result)


@tree.command()
async def loading(interaction):
    await thinking(ctx, 10)


@tree.command()
async def leaderboards(interaction):
    try:
        async with ctx.channel.typing():
            responses = await get_leaderboards(interaction)

            if len("\n".join(responses)) > 2000:
                result = ""
                for response in responses:
                    if len(result) + len(response) <= 2000:
                        result += f"{response}\n"
                    else:
                        await interaction.response.send_message(result)
                        time.sleep(1)
                        result = ""
            else:
                await interaction.response.send_message("\n".join(responses))
    except Exception as e:
        print(e)
        await interaction.response.send_message("oops, something went wrong :blush:")


@tree.command()
async def quotedump(interaction):
    msg = "suck it Tim"
    try:
        conn = sqlite3.connect(DATABASE)
        cursor = conn.cursor()
        qs = cursor.execute("SELECT * FROM quotes ORDER BY user_id").fetchall()
        conn.commit()
        conn.close()
        msg = "\n".join([f"<@{q[0]}>\t{q[1]}" for q in qs])
    except Exception as e:
        await interaction.response.send_message(e)
        return

    await interaction.response.send_message(msg)


@tree.command()
async def mysterious_merchant(interaction):
    def get_article(word):
        if word[0].lower() in ["a", "e", "i", "o", "u"]:
            return f"an {word}"
        return f"a {word}"

    class FuckingWordTracker:
        def __init__(self):
            with open("./data/item_desc.txt") as d:
                desc_lines = d.readlines()
                descriptors = [desc.strip() for desc in desc_lines]
            with open("./data/items.txt") as i:
                item_lines = i.readlines()
                items = [it.strip() for it in item_lines]
            with open("./data/merchants.txt") as m:
                merchant_lines = m.readlines()
                merchants = [mer.strip() for mer in merchant_lines]
            self.merchants = merchants
            self.items = items
            self.descriptors = descriptors
            self.used = []

        def get_descriptor(self, article=False):
            res = self.descriptors[random.randrange(len(self.descriptors) - 1)]
            self.descriptors.remove(res)
            if article:
                return f"{get_article(res)}"
            return res

        def get_item(self, article=False):
            res = self.items[random.randrange(len(self.items) - 1)]
            self.items.remove(res)
            if article:
                return f"{get_article(res)}"
            return res

        def get_merchant(self, article=False):
            res = self.merchants[random.randrange(len(self.merchants) - 1)]
            self.merchants.remove(res)
            if article:
                return f"{get_article(res)}"
            return res

        def get_item_list(self, list_len=1):
            result = []
            while len(result) <= list_len:
                result.append(f"- {self.get_descriptor()} {self.get_item()}\n")
            return "".join(result)

    try:
        words = FuckingWordTracker()
        msg = (
            f"Your tawdry little invocation summons {words.get_descriptor(True)} {words.get_merchant()}. "
            f"They stand too close to you. They offer you their paltry wares. "
            f"Type /select <item> to choose an item: \n"
            f"{words.get_item_list(list_len=5)}"
        )

        await interaction.response.send_message(msg)
    except Exception as e:
        await interaction.response.send_message(e)


@tree.command()
async def select(
    ctx, arg1: str, arg2: str = None, arg3: str = None, arg4: str = None
):  # no arrays allowed doofus
    args = [arg1, arg2, arg3, arg4]
    if len(args) == 0:
        await interaction.response.send_message("You have to select something!")
    item = " ".join(args)
    msg = f"You have selected the {item}. "
    if random.randrange(0, 10) == 0:
        msg += f"The {item} brings you peace and prosperity."
        await interaction.response.send_message(msg)
        msg = f"The {item} has broken. You shouldn't have bought it."
    else:
        msg += f"You have been cursed by the {' '.join(args)}. You shouldn't have bought it."
    await interaction.response.send_message(msg)


@tree.command()
async def sepuku(interaction):
    await interaction.response.send_message("https://giphy.com/gifs/KRY2oGS7SPvO0")


@tree.command()
async def seppuku(interaction):
    await interaction.response.send_message("https://giphy.com/gifs/KRY2oGS7SPvO0")


@tree.command()
async def die(interaction):
    await interaction.response.send_message("https://giphy.com/gifs/KRY2oGS7SPvO0")


@tree.command()
async def discipline_ryan(interaction):
    await interaction.response.send_message(f"No! Bad Ryan! Bad!")
    await interaction.response.send_message("https://imgur.com/a/21iBAu0")
