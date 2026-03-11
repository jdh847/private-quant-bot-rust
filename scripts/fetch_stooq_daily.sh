#!/usr/bin/env bash
set -euo pipefail

# Template data fetcher for daily bars using stooq.com CSV endpoints.
# You are responsible for verifying data licensing/terms for your use case.
#
# Usage:
#   ./scripts/fetch_stooq_daily.sh AAPL.US MSFT.US 7203.JP
#
# Output:
#   outputs_rust/fetched/stooq_<SYMBOL>.csv

out_dir="outputs_rust/fetched"
mkdir -p "$out_dir"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <STOOQ_SYMBOL...>"
  exit 2
fi

for sym in "$@"; do
  url="https://stooq.com/q/d/l/?s=${sym}&i=d"
  out="$out_dir/stooq_${sym}.csv"
  echo "fetch $sym -> $out"
  curl -fsSL "$url" -o "$out"
done

echo "done: $out_dir"

