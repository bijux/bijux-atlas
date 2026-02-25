# Kubernetes Install Contract

- Owner: bijux-atlas-operations
- Stability: stable

## Command

- `bijux dev atlas ops k8s apply --profile <name> --apply --allow-subprocess --allow-write`

## Rules

- Rendered manifests must exist before apply.
- Install profile must be declared in `ops/k8s/install-matrix.json`.
- Chart values must come from `ops/k8s/values/*.yaml`.
