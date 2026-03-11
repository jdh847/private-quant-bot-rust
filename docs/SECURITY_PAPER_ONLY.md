# Paper-Only Safety Model

This repo is intended to be safe to run by default. It is not a live trading system.

## Hard Guarantees

- `broker.paper_only` must be `true` (config validation rejects `false`).
- `broker.mode` supports only:
  - `sim` (default)
  - `ibkr_paper` (paper adapter)

## Hard Switches (Env Opt-In)

Even paper routes can surprise you if you accidentally point at a gateway.

To use `ibkr_paper` at all:

- set `PQBOT_ALLOW_IBKR_PAPER=1`

To allow *any* network calls (FX/IBKR HTTP):

- set `PQBOT_ALLOW_NETWORK=1`

If you do not set these, the engine will refuse to make network calls and will fall back to safe behavior.

## Audit Trail

For `run` and `demo`, the engine writes:

- `audit_snapshot.json`
- `audit_snapshot_summary.txt`

These contain:

- config file hash
- data file hashes (per market)
- key performance stats

## Token Hygiene

If you ever paste a GitHub token (PAT) or broker credential in a chat/log, assume it is compromised.

Action:

1. Revoke the token immediately in GitHub settings.
2. Rotate any dependent credentials.
3. Ensure no secrets are committed to git history.

