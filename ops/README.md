# Ops

- Owner: `bijux-atlas-operations`

## What

Canonical operational filesystem surface and SSOT entrypoint for ops workflows.

Reference contract: `ops/CONTRACT.md`.
Runbook index: `ops/INDEX.md`.

## Directory map

- `ops/stack/`: local stack dependency bring-up.
- `ops/k8s/`: chart, install profiles, and k8s-only gates.
- `ops/obs/`: observability pack, contracts, and drills.
- `ops/load/`: k6 suites, scenarios, contracts, baselines.
- `ops/datasets/`: dataset manifest, pinning, QC, promotion.
- `ops/e2e/`: composition-only scenarios over stack/obs/load/datasets.
- `atlasctl ops ...` and `make` wrappers: operator entrypoints (no direct `ops/run/` surface).
- `ops/_lib/`: shared helper libraries.
- `ops/_meta/`: ownership/surface/contracts metadata.
- `ops/_schemas/`: ops JSON schemas.
- `ops/_generated_committed/`: deterministic generated ops outputs committed to git.
- `artifacts/evidence/`: runtime evidence outputs (gitignored).
- `ops/_artifacts/`: canonical runtime artifacts root.

## Run

- `make ops-help`
- `make ops-surface`
- `make ops-layout-lint`
- `make ops-full`

Modes:
- `OPS_MODE=fast make ops-full`
- `OPS_MODE=full make ops-full`
- `OPS_DRY_RUN=1 make ops-full`
