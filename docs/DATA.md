# Data & Backtest Credibility

This project is built to make research reproducible and avoid "mystery backtests". The bundled dataset is small and synthetic; for real research you must plug in your own data pipeline.

## Data Schema

Market bars (CSV):

- Columns: `date,symbol,close,volume`
- `date`: `YYYY-MM-DD` (exchange date)
- `close`: expected to be **adjusted close** for corporate actions
- Optional: `adj_close` (if present, it overrides `close`)
- `volume`: >= 0 (0 is treated as halted/illiquid)

Industry map (CSV):

- Columns: `symbol,industry`

## Corporate Actions (Splits/Dividends)

The engine currently treats `Bar.close` as the *price series used for returns/volatility*.

- If you want split/dividend correctness, feed **adjusted close**.
- If your provider gives both, write `adj_close` and keep `close` as raw close.

## Halts / Suspensions / Missing Bars

This repo is intentionally conservative:

- If a symbol has **no bar** on a date, or `close <= 0`, orders are rejected as a data gap.
- If `volume <= 0`, orders are rejected as halted/illiquid.

See: `src/market.rs` rules.

## A-Share Rules (Simplified)

Built-in guardrails:

- `T+1` sell constraint (same-day sells rejected)
- limit-up/limit-down day guardrails (based on previous close and ~10% band)

See: `src/market.rs`.

## Trading Sessions & Holidays

Backtests are daily-bar based. The calendar is a small hard-coded holiday set for 2025-2026:

- US: NYSE-like closures
- JP: TSE-like closures
- A: China A-share closures

See: `src/calendar.rs`.

If you run outside the covered dates, you should extend the holiday lists or replace the calendar with a proper service.

## Reproducibility: Dataset Manifest

You can generate a machine-readable manifest with file hashes and date ranges:

```bash
cargo run -- dataset-manifest --config config/bot.toml --output-path data/DATASET_MANIFEST.json
```

This helps you tie results to:

- exact `config` hash
- exact `data` file hashes
- dataset date coverage

