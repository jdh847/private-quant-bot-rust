#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo run -- demo --config config/bot.toml --lang en

latest="$(cat outputs_rust/demo/LATEST_DASHBOARD.txt)"
echo "latest dashboard: $latest"

if command -v open >/dev/null 2>&1; then
  open "$latest" || true
fi

