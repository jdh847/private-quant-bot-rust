#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo run -- research \
  --config config/bot.toml \
  --output-dir outputs_rust/research \
  --markets US,A,JP \
  --short-windows 3,4,5 \
  --long-windows 7,9,11 \
  --vol-windows 5,7 \
  --top-ns 1,2 \
  --min-momentums=-0.01,0.0,0.01 \
  --strategy-plugins layered_multi_factor,momentum_guard \
  --portfolio-methods risk_parity,hrp \
  --lang en

echo "wrote: outputs_rust/research/research_leaderboard.csv"

