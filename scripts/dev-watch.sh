#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

: "${DATABASE_URL:?Set DATABASE_URL to your PostgreSQL endpoint}"
set -- --restart --stop-signal SIGTERM --stop-timeout 10s
if [[ "${DEV_WATCH_POLL:-0}" == "1" ]]; then
  set -- "$@" --poll 1s
fi
set -- "$@" --watch src --watch examples --watch migrations \
  --watch Cargo.toml --watch Cargo.lock --watch build.rs
if [[ "${DEV_TLS:-1}" == "1" ]]; then
  set -- "$@" -- cargo run --locked --example dev-server
else
  set -- "$@" -- cargo run --locked
fi
exec watchexec "$@"
