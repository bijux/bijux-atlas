# Ops Map

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/inventory/pillars.json`, `ops/inventory/surfaces.json`
- Contract references: `OPS-ROOT-023`, `OPS-INV-PILLARS-001`, `OPS-INV-001`

## What

This page maps each ops pillar to its single docs entry page.

## Why

Operations guidance must have one stable entry point per pillar so runbooks, contracts, and tooling all resolve to the same place.
This keeps `Release-indexed` operational flows anchored to one entry page per pillar instead of spreading them across duplicate walkthroughs.

## Pillars

| Pillar | Ops Surface | Docs Entry |
|---|---|---|
| `inventory` | `ops/inventory` | `../reference/ops-surface.md` |
| `schema` | `ops/schema` | `../reference/schemas.md` |
| `datasets` | `ops/datasets` | `dataset-workflow.md` |
| `e2e` | `ops/e2e` | `e2e/index.md` |
| `env` | `ops/env` | `config.md` |
| `stack` | `ops/stack` | `run-locally.md` |
| `k8s` | `ops/k8s` | `k8s/index.md` |
| `load` | `ops/load` | `load/index.md` |
| `observe` | `ops/observe` | `observability/index.md` |
| `report` | `ops/report` | `release/index.md` |

## How To Verify

```bash
cargo test -q -p bijux-dev-atlas --test docs_ops_coherence_contracts -- --nocapture
```

Expected output: each ops pillar resolves to exactly one docs entry page and all mapped paths exist.
