# Kubernetes Upgrade Contract

- Owner: bijux-atlas-operations
- Stability: stable

## Command

- `bijux dev atlas ops install --profile <name> --allow-subprocess --allow-write --allow-network`

## Rules

- Upgrade is chart-version pinned; floating chart references are forbidden.
- CRD and values schema compatibility must pass before apply.
- Conformance checks must run after upgrade.
