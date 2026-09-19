#! /bin/bash
# Build the release bot binary and stage it in dist/ under the name the deploy
# expects. x86_64 only, and just the bot — dist/wan-party-bot-arm64 and
# dist/trigger_poll-x86_64 are built by hand.
set -euo pipefail

cargo zigbuild --release --target x86_64-unknown-linux-musl --bin wan-party-bot

mkdir -p dist
cp target/x86_64-unknown-linux-musl/release/wan-party-bot dist/wan-party-bot-x86_64
