> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Kubernetes Rollback Contract

- Owner: bijux-atlas-operations
- Stability: stable

## Command

- `bijux dev atlas ops stack down --profile <name> --allow-subprocess --allow-write --allow-network`

## Rules

- Rollback uses immutable pins from inventory and stack manifests.
- Rollback guidance must be emitted on failed apply paths.
- Rollback output is included in ops report artifacts.
