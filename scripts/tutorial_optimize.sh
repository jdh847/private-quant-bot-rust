#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo run -- optimize \
  --config config/bot.toml \
  --output-dir outputs_rust/optimize \
  --train-days 12 \
  --test-days 5 \
  --short-windows 3,4,5 \
  --long-windows 7,9,11 \
  --vol-windows 5,7 \
  --top-ns 1,2 \
  --min-momentums=-0.01,0.0,0.01 \
  --strategy-plugins layered_multi_factor,momentum_guard \
  --portfolio-methods risk_parity,hrp \
  --lang en

echo "wrote: outputs_rust/optimize/walk_forward_folds.csv"

