# Ops Contract (SSOT)

- Owner: `bijux-atlas-operations`
- Contract version: `1.0.0`

## Purpose

`ops/` is the single source of truth for operational topology, executable entrypoints, and contract-governed artifacts.

## Invariants

- Stable operator entrypoints are Make targets listed in `ops/INDEX.md` and declared in `ops/_meta/surface.json`.
- `ops/run/` is the only executable entrypoint subtree; files there are thin wrappers that call `ops/_lib/` and domain scripts.
- Shared shell/python helpers that are not operator entrypoints live in `ops/_lib/` or domain-local `scripts/` directories.
- Canonical domain ownership has no overlap:
  - `ops/stack/` local dependency bring-up
    - owns canonical fault injection API at `ops/stack/faults/inject.sh`
  - `ops/k8s/` chart, profiles, and k8s gates
  - `ops/obs/` observability pack, contracts, drills
  - `ops/load/` k6 suites and baselines
  - `ops/datasets/` dataset manifests, pinning, QC, promotion
  - `ops/e2e/` composition-only scenarios across domains
    - `ops/e2e/k8s/tests/` only wrapper entry scripts, not invariant test ownership
- Generated outputs under `ops/_generated/` only.
- Runtime artifacts write under `ops/_artifacts/` only, unless allowlisted in `configs/ops/artifacts-allowlist.txt`.
- JSON schemas for ops manifests live under `ops/_schemas/`.
- Symlinked domain directories inside `ops/` are forbidden.
- Canonical pinned tool versions live in `configs/ops/tool-versions.json`.

## Stable vs Generated

Stable (versioned by review):
- `ops/CONTRACT.md`, `ops/INDEX.md`
- `ops/_meta/*.json`
- domain definitions and tests under canonical subtrees

Generated (rebuildable):
- `ops/_generated/**`

Runtime artifacts (ephemeral evidence):
- `ops/_artifacts/**`

## Artifact Rules

- Scripts must not write to repo-root `artifacts/` directly.
- Legacy compatibility, if needed, is via symlink: `artifacts/ops -> ops/_artifacts`.
- Any explicit exception must be listed in `configs/ops/artifacts-allowlist.txt`.
