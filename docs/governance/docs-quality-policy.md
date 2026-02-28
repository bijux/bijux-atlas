# Docs Quality Policy

## Purpose
- Owner: `docs-governance`
- Stability: `stable`

Define repository-wide documentation quality rules and enforcement boundaries.

## Stability
- v0.1.0 stable scope.
- Canonical policy source: `configs/docs/quality-policy.json`.
- Enforcement surface: `bijux dev atlas docs validate` and `docs registry validate`.

## Guarantees
- Policy checks are deterministic and run in CI.
- Forbidden terminology and naming violations fail validation.
- Stale and orphan documentation is surfaced with stable error codes.
