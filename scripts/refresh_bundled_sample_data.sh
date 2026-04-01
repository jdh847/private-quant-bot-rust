#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo run --bin private_quant_bot -- gen-synth-data \
  --output-dir data \
  --start-date 2024-01-02 \
  --end-date 2026-03-31 \
  --seed 42 \
  --us-symbols 12 \
  --a-symbols 12 \
  --jp-symbols 12 \
  --industries-per-market 6 \
  --force

echo "Bundled sample data refreshed under $ROOT/data"
