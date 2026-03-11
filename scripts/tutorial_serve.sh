#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Starting localhost server for: outputs_rust/demo"
echo "If this environment blocks binding ports, run it locally on your machine."
cargo run -- serve --root outputs_rust/demo --bind 127.0.0.1:8787 --lang en

