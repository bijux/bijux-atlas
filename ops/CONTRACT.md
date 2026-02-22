# Ops Contract (SSOT)

- Owner: `bijux-atlas-operations`
- Contract version: `1.0.0`

## Purpose

`ops/` is the single source of truth for operational topology, manifests/contracts, and contract-governed artifacts.

## Invariants

- Stable operator entrypoints are Make targets listed in `ops/INDEX.md` and declared in `ops/_meta/surface.json`.
- Operator entrypoints are `atlasctl ops ...` commands and thin `make` wrappers; `ops/` shell scripts are implementation adapters only.
- Shared shell/python helpers that are not operator entrypoints live in `ops/_lib/` or domain-local `scripts/` directories.
- Canonical domain ownership has no overlap:
  - `ops/stack/` local dependency bring-up
    - owns canonical fault injection API at `packages/atlasctl/src/atlasctl/commands/ops/stack/faults/inject.py`
  - `ops/k8s/` chart, profiles, and k8s gates
  - `ops/obs/` observability pack, contracts, drills
  - `ops/load/` k6 suites and baselines
  - `ops/datasets/` dataset manifests, pinning, QC, promotion
  - `ops/e2e/` composition-only scenarios across domains
    - `ops/e2e/k8s/tests/` only wrapper entry scripts, not invariant test ownership
- Deterministic generated outputs under `ops/_generated_committed/` only.
- Runtime evidence outputs under `artifacts/evidence/` only.
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
- `ops/_generated_committed/**`
- `ops/_examples/**`
- committed generated files policy:
  - commit: `ops/_examples/report.example.json`
  - commit: `ops/_examples/report.unified.example.json`
  - commit: `docs/_generated/ops-surface.md`
  - commit: `docs/_generated/ops-contracts.md`
  - commit: `docs/_generated/ops-schemas.md`
  - non-listed generated outputs are ephemeral and must be cleaned.

Runtime artifacts (ephemeral evidence):
- `ops/_artifacts/**`
- `artifacts/evidence/**`

## Artifact Rules

- Runtime evidence must write under `artifacts/evidence/**`.
- Runtime artifacts (non-evidence) write under `ops/_artifacts/**`.
- `ops/_generated/` is reserved for static/generated contract assets only (no run-scoped evidence).
- Any explicit exception must be listed in `configs/ops/artifacts-allowlist.txt`.

## Schema Evolution

- All ops schemas are versioned under `ops/_schemas/`.
- Backward-compatible changes are additive and keep `v1` stable.
- Breaking changes require a new schema version and migration notes in `ops/CONTRACT.md`.
- `make ops-contracts-check` is the required gate for schema conformance.

## Compatibility Guarantee

- Ops manifest `v1` contracts remain stable for automation and CI consumers.
- Existing required keys must not be removed in `v1`.
- New keys in `v1` must be optional unless a version bump is performed.

## Deprecation Policy

- Script deprecations must preserve Make target behavior for at least one release window.
- Deprecated scripts must print migration guidance and point to canonical Make targets.
- Removal requires updating `ops/_meta/surface.json`, `ops/INDEX.md`, and generated ops docs.

## Naming and Directory Policy

- Script and manifest names must be durable nouns with qualifiers; temporal/task naming is forbidden.
- Empty directories are forbidden unless they contain `INDEX.md` documenting intent.
