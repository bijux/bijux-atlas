# Ops Lifecycle Contracts

- Owner: `docs-governance`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define the executable lifecycle checks for kind-backed upgrade and rollback simulation.

## Contract IDs

- `OPS-LIFE-001`: upgrade `profile-baseline` succeeds and writes `ops-upgrade.json`.
- `OPS-LIFE-002`: rollback `profile-baseline` succeeds and writes `ops-rollback.json`.
- `OPS-LIFE-003`: upgrade `ci` succeeds and writes `ops-upgrade.json`.
- `OPS-LIFE-004`: rollback `ci` succeeds and writes `ops-rollback.json`.
- `OPS-LIFE-005`: upgrade `offline` succeeds and writes `ops-upgrade.json`.
- `OPS-LIFE-006`: rollback `offline` succeeds and writes `ops-rollback.json`.
- `OPS-LIFE-007`: upgrade `perf` succeeds and writes `ops-upgrade.json`.
- `OPS-LIFE-008`: rollback `perf` succeeds and writes `ops-rollback.json`.

## Required evidence

- `ops-upgrade.json`
- `ops-rollback.json`
- `ops-lifecycle-summary.json`

## Required checks

- Immutable field compatibility must stay safe.
- Service names, selectors, and ports must remain stable.
- PVC definitions must remain stable when persistence is present.
- Ingress host shape, network policy defaults, HPA defaults, and required env keys must remain stable.
- Rollout history and pod restart counts must be captured for every upgrade and rollback run.

## Reproduce locally

```bash
bijux dev atlas ops helm install --profile profile-baseline --cluster kind --chart-source previous --allow-subprocess --allow-write --allow-network --format json
bijux dev atlas ops helm upgrade --profile profile-baseline --cluster kind --to current --allow-subprocess --allow-write --allow-network --format json
bijux dev atlas ops helm rollback --profile profile-baseline --cluster kind --to previous --allow-subprocess --allow-write --allow-network --format json
```
