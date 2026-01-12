blacklist = [
  "sigh-politics",
  "bible",
  "wanglicanism",
  "formative movie crushes of the youthful era",
  "dads"
]

def is_blacklisted_channel(channel_name):
  return channel_name.lower() in blacklist