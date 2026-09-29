#! /bin/bash
# Runs on the server, piped over ssh by .github/workflows/deploy.yml after CI has
# uploaded freshly built binaries as dist/*.new. The server is too small to build
# them itself.
set -euo pipefail

cd ~/wan-party-bot
cp wanparty.db ../backups/wanparty.db-$(date '+%Y-%m-%d%H%M%S')
git fetch
git reset origin/master --hard

# Swap the new binaries in. Untracked, so the reset above leaves them alone; mv
# within one filesystem is atomic, so nothing ever runs a half-copied file.
for bin in wan-party-bot-x86_64 trigger_poll-x86_64; do
  chmod +x "dist/$bin.new"
  mv "dist/$bin.new" "dist/$bin"
done

# -n: fail the deploy instead of hanging if sudo ever wants a password.
sudo -n systemctl restart partybot
# see /etc/systemd/system/partybot.service
# logs: journalctl -u partybot -f
