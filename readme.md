# Wan Party Discord Bot

You have chosen, or been chosen, to relocate to one of our finest remaining codebases. I thought so much of WAN Party Bot that I elected to establish my Administration here, in this repo so thoughtfully provided by Our Benefactors. I have been proud to call WAN Party Bot my home. And so, whether you are here to stay, or passing through on your way to parts unknown, welcome to WAN Party Bot. It's safer here.

- just for fun Discord bot to do fun things just for fun
- anything pushed to `master` gets deployed

## Forthcoming Features

- ability to update crontab

- rewritten in Rust (the original Python implementation has been removed)

## Running locally

The bot is a Rust/`serenity` application. Two binaries mirror the two original
Python entrypoints:

- `wan-party-bot` — the bot itself (was `main.py`)
- `trigger_poll` — post the weekly games poll once, then exit (was `trigger_poll.py`)

```zsh
# build
cargo zigbuild --release --target x86_64-unknown-linux-musl

# run the bot (reads ANTHROPIC_API_KEY / JEV_API_KEY / GIPHY_TOKEN from the
# environment too)
DISCORD_TOKEN=TOKEN_HERE cargo run --release --bin wan-party-bot

# post the weekly poll and exit
DISCORD_TOKEN=TOKEN_HERE cargo run --release --bin trigger_poll
```

It still reads the `data/`, `corpora/`, and `wanparty.db` files by relative path,
so run it from the repo root.

## AI routing

When someone @-mentions the bot, TypeSafe's Jev (`src/jev.rs`) gets the message
first and decides two things in one call: is this a yes/no question, and if so is
the answer yes. It sees the message, whatever is being replied to, and the same
long-term memory digest Claude gets, so it can answer a question that turns on who
someone is. A confident yes/no question is answered on the spot; everything
else goes to Claude (`src/chat.rs`) as before. Without `JEV_API_KEY` — or on
any Jev failure — every message falls through to Claude, so the bot keeps working.

Claude failures react with each feature's usual shrug emoji, except billing
failures (out of credits, dead card), which react 💰.
