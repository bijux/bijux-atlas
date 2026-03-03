# Ops Directory Necessity

- Owner: `bijux-atlas-operations`
- Purpose: document why each canonical ops directory must exist.
- Consumers: `checks_ops_minimalism_and_deletion_safety`

## Canonical Directories

- `ops/datasets`
- `ops/e2e`
- `ops/env`
- `ops/inventory`
- `ops/k8s`
- `ops/load`
- `ops/observe`
- `ops/report`
- `ops/schema`
- `ops/stack`

## Deletion Safety Notes

Deleting a canonical directory breaks inventory mapping, schema coverage, and release evidence continuity.
