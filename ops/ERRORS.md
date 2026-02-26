# Ops Errors

- Authority Tier: `machine`
- Audience: `operators`
## Scope

Stable error identifiers for `bijux dev atlas ops ...` command surfaces.

## Contract

- Error identifiers are stable strings intended for CI parsing and troubleshooting.
- Human-readable messages may evolve, but identifiers should not churn without a migration update.
- Ops commands are capability-gated; missing flags should fail with explicit effect requirements.

## Namespace Convention

- Error IDs follow `OPS_<DOMAIN>_<CODE>`.
- `<DOMAIN>` is uppercase and intent-based (for example `CONTRACT`, `TOOL`, `RENDER`, `INSTALL`, `STATUS`, `PINS`, `REPORT`).
- `<CODE>` is uppercase and stable for machine consumers.
- Deprecated IDs require a compatibility window and explicit migration note.

## Current Error IDs

- `OPS_CONTRACT_ERROR`: Ops SSOT contract violation or missing required inputs
- `OPS_TOOL_ERROR`: Required external tool missing or version contract mismatch
- `OPS_RENDER_ERROR`: Deterministic render/check contract failed
- `OPS_INSTALL_ERROR`: Install/apply operation refused or failed
- `OPS_STATUS_ERROR`: Status collection failed
- `OPS_PINS_ERROR`: Pins validation or update contract failed
- `OPS_REPORT_ERROR`: Report generation or artifact write failed

## Capability Gate Failures

These are typically emitted as command errors with explicit messages:

- missing `--allow-subprocess`
- missing `--allow-write`
- missing `--allow-network`

The presence of a capability gate error indicates the command was invoked in safe mode without the
required effect enabled.
