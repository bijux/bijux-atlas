# Ops Contract (SSOT)

- Owner: `bijux-atlas-operations`
- Contract version: `1.1.0`
- kind-cluster-contract-hash: `b7cbaefe788fae38340ef3aa0bc1b79071b8da6f14e8379af029ac1a3e412960`

## Purpose

`ops/` is the source of truth for operational inventory, schemas, environment overlays, observability configuration, load profiles, and end-to-end manifests.
`bijux-dev-atlas` executes generation, validation, and orchestration against this contract.
Ops is specification-only. Runtime behavior and orchestration logic live in `crates/bijux-dev-atlas`.

## Ownership

- `ops/` stores specifications and contracts only.
- Operational behavior is owned by `bijux-dev-atlas` and routed through `bijux dev atlas ops ...`.
- New shell, python, or executable behavior under `ops/` is forbidden.
- Fixture-like test helpers are allowed only as non-executable data under explicit fixture paths.
- `makefiles/ops.mk` is a thin delegator to `bijux dev atlas ...` commands.
- CI workflows must invoke ops behavior through `make` wrappers or `bijux dev atlas ops ...` commands only.

## Evolution Policy

- Schemas under `ops/schema/` are versioned APIs and require explicit compatibility handling.
- Release pins are immutable after release publication; changes require a promoted replacement.
- `_generated/` is ephemeral output only and must never be edited manually.
- `_generated.example/` is curated evidence and is the only committed generated surface.
- Naming uses intent nouns and canonical names. Use `observe` as the canonical observability domain name.
- Compatibility migrations must be timeboxed and include explicit cutoff dates.
- Legacy shell compatibility deadline: 2026-06-30.

## Canonical Tree

- `ops/inventory/`: ownership, command surface, namespaces, toolchain, release pins, image pins, drill catalog.
- `ops/schema/`: schema source of truth for inventory and reports.
- `ops/env/`: overlays for `base`, `dev`, `ci`, `prod`.
- `ops/observe/`: alert rules, dashboard definitions, telemetry wiring.
- `ops/load/k6/`: load manifests, suites, thresholds, and query packs.
- `ops/e2e/`: datasets, manifests, fixtures, expectations, and suites.
- `ops/_generated/`: runtime generated outputs.
- `ops/_generated.example/`: committed generated examples only.
- Canonical directory budget: keep the top-level canonical tree intentionally small; additions require contract updates and ownership review.

## Invariants

- Command surface metadata is declared in `ops/inventory/surfaces.json`.
- Ownership metadata is declared in `ops/inventory/owners.json`.
- Schemas live only under `ops/schema/`.
- Generated runtime outputs are written only under `ops/_generated/`.
- Committed generated outputs are written only under `ops/_generated.example/`.
- Runtime evidence and artifacts are written under `artifacts/`.
- Symlinked directories under `ops/` are forbidden unless explicitly allowlisted.
- Executable-bit files under `ops/` are forbidden.
- `.sh` and `.py` files under `ops/` are forbidden except explicit fixture allowlist paths.

## Stable vs Generated

Stable (reviewed):
- `ops/CONTRACT.md`, `ops/INDEX.md`
- inventory documents under `ops/inventory/`
- schema documents under `ops/schema/`
- env, observe, load, and e2e source manifests

Generated (rebuildable):
- `ops/_generated/**`
- `ops/_generated.example/**` for committed examples only
- `ops/_generated.example/**`

Runtime outputs (ephemeral):
- `artifacts/**`

## Schema Evolution

- Additive updates stay backward-compatible within a major version.
- Breaking updates require a schema version bump and migration notes in this contract.
- Contract conformance is gated by `bijux dev atlas` checks and CI suites.

## Naming Policy

- Names use durable intent-focused nouns.
- Temporary or timeline-oriented naming is forbidden for committed paths and metadata keys.
