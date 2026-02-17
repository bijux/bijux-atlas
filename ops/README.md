# Ops Layout

- Owner: `bijux-atlas-operations`

## What

Defines the canonical operations filesystem surface under `ops/`.

## Why

Keeps runtime assets, drills, and deployment manifests in one stable location.

## Scope

- `ops/e2e/`: local and CI end-to-end stack assets and k8s tests
- `ops/load/`: k6 suites, load manifests, baselines, and reports
- `ops/observability/`: dashboards, alerts, observability contracts, and smoke tooling
- `ops/openapi/`: versioned OpenAPI snapshots and drift artifacts

## Non-goals

Does not replace `docs/operations/` narrative and runbook documentation.

## Contracts

- All runnable ops workflows must be exposed as `make ops-*` targets.
- Ops tool versions are pinned in `configs/ops/tool-versions.json`.

## Failure modes

Divergence between `ops/` assets and `docs/operations/` instructions causes failed drills.

## How to verify

```bash
make ops-tools-check
make ops-k8s-tests
make ops-load-smoke
```

## See also

- `docs/operations/INDEX.md`
- `docs/development/repo-surface.md`
- `configs/ops/README.md`
