#!/usr/bin/env bash
set -euo pipefail

# TEMPLATE ONLY. This script downloads daily bars via stooq.com CSV endpoints.
#
# Important:
# - This repo does NOT endorse any vendor as an "official" data source.
# - You are responsible for verifying licensing/terms and ensuring compliance.
# - Do NOT redistribute downloaded data unless you have explicit rights.
# - Data quality/adjustments (splits/dividends) are vendor-dependent.
#
# If you plan to use adjusted prices, transform data into the engine schema and set `adj_close`.
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

echo "NOTE: template-only downloader. verify licensing/terms before using."

for sym in "$@"; do
  url="https://stooq.com/q/d/l/?s=${sym}&i=d"
  out="$out_dir/stooq_${sym}.csv"
  echo "fetch $sym -> $out"
  curl -fsSL "$url" -o "$out"
done

echo "done: $out_dir"
