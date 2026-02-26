# Ops Directory Necessity

- Owner: `bijux-atlas-operations`
- Purpose: `declare why each canonical ops directory exists and what breaks if removed`
- Consumers: `checks_ops_minimalism_and_deletion_safety`

## Canonical Directories

- `ops/datasets`: dataset contracts, fixture governance, and dataset lifecycle policies
- `ops/e2e`: end-to-end suite definitions, invariants, and execution contracts
- `ops/env`: environment-specific config overlays and environment contracts
- `ops/inventory`: authority graph inputs, ownership, surfaces, and governance registries
- `ops/k8s`: install matrices, chart values, rollout safety, and cluster test manifests
- `ops/load`: load scenarios, thresholds, suites, and generated summaries
- `ops/observe`: observability contracts, drills, dashboards, and signal goldens
- `ops/report`: readiness, historical comparison, and evidence bundle contracts
- `ops/schema`: schemas and compatibility locks for ops JSON contracts
- `ops/stack`: stack profiles, local cluster topology, and version manifests

## Deletion Safety Notes

- Removing a canonical directory requires updating `ops/CONTRACT.md`, `ops/inventory/authority-index.json`, and `ops/inventory/contracts-map.json`.
- Any deletion must preserve check coverage (`checks_*`) or replace it with an equivalent stronger check in the same commit.
- Generated and example evidence may be removed only after their producer and consuming checks are updated together.
