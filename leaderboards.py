import asyncio
import time

from thinking import thinking
from lexical_analysis import lexical_analysis

async def get_leaderboards(ctx):
  thinking_task = asyncio.create_task(thinking(ctx))
  fetching_task = asyncio.create_task(fetch_leaderboards(ctx))
  responses = ['uhoh']
  try:
    responses = await fetching_task
  except Exception as e:
    print(e)
  finally:
    thinking_task.cancel()

  return responses

async def sort_leaderboards(users):
  sorted_users = sorted(users.values(), key=lambda x: x["message_count"], reverse=True)
  return sorted_users


async def fetch_leaderboards(ctx):
  print('fetching leaderboards')
  users = {}
  messages = ctx.channel.history(limit=10000)
  async for msg in messages:
      name = msg.author.name
      word_count = len(msg.content.split(" "))
      if name is None:
          continue

      if name in users:
          users[name]["message_count"] += 1
          users[name]["word_count"] += word_count

      else:
          users[name] = {
              "name": name,
              "message_count": 1,
              "word_count": word_count,
          }
  channel_name = getattr(ctx.channel, 'name', 'this channel')
  responses = [f'**Message stats for "{channel_name}"**']
  sorted_users = await sort_leaderboards(users)

  print('analyzing chat data')
  for user_info in sorted_users:
      message_count = user_info["message_count"]
      avg_word_count = int(user_info["word_count"] / message_count)
      name = user_info["name"]
      grade = lexical_analysis(user_info)
      responses.append(f'> *{name}*: **{message_count}** messages, avg length: **{avg_word_count}** words, :brain: estimate: **{grade}**')

  return responses

