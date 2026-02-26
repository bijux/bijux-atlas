# Ops Artifacts

- Authority Tier: `machine`
- Audience: `operators`
## Purpose
Define the canonical runtime artifact root and path layout for ops workflows.

## Authority
- Authored policy: `ops/ARTIFACTS.md`
- Runtime outputs: `artifacts/**` (gitignored)
- Curated committed examples: `ops/_generated.example/**`

## Canonical root
- Runtime outputs must be written under `artifacts/`.
- Dev-atlas ops runs use `artifacts/atlas-dev/<domain>/<run-id>/...`.
- Ops workflows must not write runtime outputs under `ops/`, except ephemeral runtime files under `ops/_generated/`.

## Domain layout examples
- Stack: `artifacts/atlas-dev/ops/<run-id>/stack/`
- Datasets: `artifacts/atlas-dev/ops/<run-id>/datasets/`
- Kubernetes: `artifacts/atlas-dev/ops/<run-id>/k8s/`

## Migration
- Legacy `ops/_artifacts/**` references are retired.
- Cutoff date: February 26, 2026.
- Any new `ops/_artifacts` path in docs, suites, contracts, or runtime outputs is contract drift.
