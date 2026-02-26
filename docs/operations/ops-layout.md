# Ops Layout

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Canonical operational filesystem layout.

## Directory map

- `ops/stack/`: stack manifests and bootstrap scripts.
- `ops/e2e/`: harness and test runners only.
- `ops/k8s/`: charts, values, and tests.
- `ops/load/`: k6 suites and scoring.
- `ops/observe/`: dashboards, alerts, and contracts.
- `ops/datasets/`, `ops/datasets/fixtures/`: dataset and fixture assets.
- bijux dev atlas ops helper assets: `crates/bijux-dev-atlas/src/commands/ops/runtime_modules/assets/lib/`.

## Run full stack

```bash
$ make ops-full
```

Canonical meta target: `ops-full`.

## See also

- [Operations Index](INDEX.md)
- [Full Stack Local](full-stack-local.md)
