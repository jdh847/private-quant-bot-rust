# Data (Sample Only)

This repository includes a tiny, **synthetic** daily-bar dataset under `data/` so you can run the bot end-to-end without downloading anything.

## What It Is

- Daily bars: `date,symbol,close,volume`
- Minimal industry maps: `symbol,industry`
- Markets: `US`, `A` (A-share), `JP`

The sample data is **not** meant to be realistic, complete, or tradable. It exists only to make tests, demos, and tutorials reproducible.

## Generate Your Own Synthetic Dataset

If you want a bigger reproducible dataset (still synthetic), use:

```bash
cargo run -- gen-synth-data --output-dir data_synth --seed 42 --us-symbols 12 --a-symbols 12 --jp-symbols 12 --force
```

Then point `config/bot.toml` to `data_synth/*.csv`.

## What The Engine Expects

- `close` should be *adjusted close* if you have corporate actions (splits/dividends).
- Optional column supported: `adj_close` (if present, it is used instead of `close`).
- Missing rows for a symbol on a trading day are treated as **missing data** (orders will be rejected).

## License / Ownership

The sample dataset in this folder is intended to be treated as **generated / synthetic**.
If you replace it with real market data, make sure you have the right to store and redistribute it.
