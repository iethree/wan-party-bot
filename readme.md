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
cargo build --release

# run the bot (reads ANTHROPIC_API_KEY / GIPHY_TOKEN from the environment too)
DISCORD_TOKEN=TOKEN_HERE cargo run --release --bin wan-party-bot

# post the weekly poll and exit
DISCORD_TOKEN=TOKEN_HERE cargo run --release --bin trigger_poll
```

It still reads the `data/`, `corpora/`, and `wanparty.db` files by relative path,
so run it from the repo root.
