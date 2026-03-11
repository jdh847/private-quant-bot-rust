# Security Policy

## Scope

This project is paper-trading first, but still processes trading logic and credentials in local configs.

## Reporting a vulnerability

Please do not open public issues for security vulnerabilities.

Report details privately with:

- vulnerable component/file
- reproduction steps
- impact assessment
- suggested mitigation (if available)

Maintainers will acknowledge receipt and provide status updates.

## Hardening guidance

- This repository is designed to be **paper-only**. `broker.paper_only=false` is rejected by config validation.
- IBKR paper adapter is disabled by default. To enable it you must set:
  - `PQBOT_ALLOW_IBKR_PAPER=1`
  - and (if `ibkr.enabled=true` and `ibkr.dry_run=false`) `PQBOT_ALLOW_NETWORK=1`
- Live FX is disabled by default. If you enable it (`fx.live_enabled=true`), network calls require `PQBOT_ALLOW_NETWORK=1`.
- Avoid committing account IDs, API keys, or private gateway endpoints.
- Prefer localhost for broker gateways when possible.

More details: `docs/SECURITY_PAPER_ONLY.md`.
