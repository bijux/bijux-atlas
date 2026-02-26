# Evidence Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines where runtime evidence is written and how operators should read, clean, and share run reports.

## Why

Separates ephemeral run outputs from committed source artifacts so git state and review diffs stay deterministic.

## Contracts

- Canonical runtime evidence root is `artifacts/evidence/`.
- Make lane reports write to `artifacts/evidence/make/<lane>/<run_id>/report.json`.
- Root-local pointer files are:
  - `artifacts/runs/latest-run-id.txt`
  - `artifacts/evidence/root-local/latest-run-id.txt`
- Committed generated artifacts remain under `ops/_generated.example/`.

## Where To Find Reports

- Latest unified lane report:
  - `artifacts/evidence/make/<run_id>/unified.json`
- Root-local summary:
  - `artifacts/evidence/make/root-local/<run_id>/summary.md`
- PR markdown summary:
  - `make evidence/pr-summary`

## Failure modes

- Writing runtime reports under `ops/` causes contract drift and lint failures.
- Reading stale run IDs can produce incorrect triage links.

## How to verify

```bash
make evidence/check
make evidence/open AREA=make
make evidence/clean
```

Expected output: evidence schema check passes, open prints an existing evidence path, and cleanup reports retained/pruned runs.
