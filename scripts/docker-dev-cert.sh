#!/usr/bin/env bash
set -euo pipefail
cd /app

# Docker can create missing bind-mount directories as root on Linux. Use the
# checkout owner's identity so native development can read the same private key.
owner=$(stat -c '%u:%g' /workspace)
mkdir -p .dev/certs
chown "$owner" .dev .dev/certs
chmod 700 .dev .dev/certs
for file in .dev/certs/localhost.pem .dev/certs/localhost-key.pem; do
  if [[ -f "$file" ]]; then
    chown "$owner" "$file"
    chmod 600 "$file"
  fi
done

exec setpriv --reuid "${owner%:*}" --regid "${owner#*:}" --clear-groups \
  bash scripts/dev-cert.sh generate
