#! /bin/bash
# Build the release Linux binaries and stage them in dist/ under the names the
# server expects. CI runs this on every push to master (.github/workflows/deploy.yml);
# run it locally to check a build. dist/ is gitignored — binaries aren't committed.
set -euo pipefail

cargo zigbuild --release --target x86_64-unknown-linux-musl --bins

mkdir -p dist
for bin in wan-party-bot trigger_poll; do
  cp "target/x86_64-unknown-linux-musl/release/$bin" "dist/$bin-x86_64"
done
