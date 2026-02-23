# Ops Contract (SSOT)

- Owner: `bijux-atlas-operations`
- Contract version: `1.1.0`
- kind-cluster-contract-hash: `b7cbaefe788fae38340ef3aa0bc1b79071b8da6f14e8379af029ac1a3e412960`

## Purpose

`ops/` is the source of truth for operational inventory, schemas, environment overlays, observability configuration, load profiles, and end-to-end manifests.
`atlasctl` executes generation, validation, and orchestration against this contract.

## Canonical Tree

- `ops/inventory/`: ownership, command surface, namespaces, toolchain, release pins, image pins, drill catalog.
- `ops/schema/`: schema source of truth for inventory and reports.
- `ops/env/`: overlays for `base`, `dev`, `ci`, `prod`.
- `ops/observe/`: alert rules, dashboard definitions, telemetry wiring.
- `ops/load/k6/`: load manifests, suites, thresholds, and query packs.
- `ops/e2e/`: datasets, manifests, fixtures, expectations, and suites.
- `ops/_generated/`: runtime generated outputs.
- `ops/_generated.example/`: committed generated examples only.

## Invariants

- Command surface metadata is declared in `ops/inventory/surfaces.json`.
- Ownership metadata is declared in `ops/inventory/owners.json`.
- Schemas live only under `ops/schema/`.
- Generated runtime outputs are written only under `ops/_generated/`.
- Committed generated outputs are written only under `ops/_generated.example/`.
- Runtime evidence and artifacts are written under `artifacts/`.
- Symlinked directories under `ops/` are forbidden unless explicitly allowlisted.

## Stable vs Generated

Stable (reviewed):
- `ops/CONTRACT.md`, `ops/INDEX.md`
- inventory documents under `ops/inventory/`
- schema documents under `ops/schema/`
- env, observe, load, and e2e source manifests

Generated (rebuildable):
- `ops/_generated/**`
- `ops/_generated.example/**` for committed examples only
- `ops/_examples/**`

Runtime outputs (ephemeral):
- `artifacts/**`

## Schema Evolution

- Additive updates stay backward-compatible within a major version.
- Breaking updates require a schema version bump and migration notes in this contract.
- Contract conformance is gated by atlasctl checks and CI suites.

## Naming Policy

- Names use durable intent-focused nouns.
- Temporary or timeline-oriented naming is forbidden for committed paths and metadata keys.
