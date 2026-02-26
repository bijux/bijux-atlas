# Ops Contract (SSOT)

- Owner: `bijux-atlas-operations`
- Authority Tier: `machine`
- Audience: `mixed`
- Contract version: `1.1.0`
- kind-cluster-contract-hash: `b7cbaefe788fae38340ef3aa0bc1b79071b8da6f14e8379af029ac1a3e412960`

## Purpose

`ops/` is the source of truth for operational inventory, schemas, environment overlays, observability configuration, load profiles, and end-to-end manifests.
`bijux-dev-atlas` executes generation, validation, and orchestration against this contract.
Ops is specification-only. Runtime behavior and orchestration logic live in `crates/bijux-dev-atlas`.

## Authority Hierarchy

- Authoritative order is strict: `ops/inventory/**` (registry truth) -> `ops/schema/**` (validation contracts) -> `docs/**` (human explanation).
- When two artifacts overlap semantically, the higher level in this hierarchy is authoritative and the lower level must reference it.
- `ops/inventory/contracts-map.json` is the canonical authority index for inventory artifacts; `ops/inventory/contracts.json` is a generated mirror only.
- `REQUIRED_FILES.md` is a presence contract only; field semantics are defined by `ops/schema/meta/required-files-contract.schema.json`.

## Ownership

- `ops/` stores specifications and contracts only.
- Operational behavior is owned by `bijux-dev-atlas` and routed through `bijux dev atlas ops ...`.
- New shell, python, or executable behavior under `ops/` is forbidden.
- No `Makefile` is allowed under `ops/`; make behavior is owned by root `makefiles/` wrappers.
- Fixture-like test helpers are allowed only as non-executable data under explicit fixture paths.
- `makefiles/ops.mk` is a thin delegator to `bijux dev atlas ...` commands.
- CI workflows must invoke ops behavior through `make` wrappers or `bijux dev atlas ops ...` commands only.

## Evolution Policy

- Schemas under `ops/schema/` are versioned APIs and require explicit compatibility handling.
- Release pins are immutable after release publication; changes require a promoted replacement.
- `_generated/` is ephemeral output only and must never be edited manually.
- `_generated.example/` is curated evidence and is the only committed generated surface.
- Naming uses intent nouns and canonical names. Use `observe` as the canonical observability domain name; `obs` is forbidden.
- Compatibility migrations must be timeboxed and include explicit cutoff dates.
- Legacy shell compatibility deadline: 2026-06-30.

## Canonical Tree

- `ops/inventory/`: ownership, command surface, namespaces, toolchain, release pins, image pins, drill catalog.
- `ops/schema/`: schema source of truth for inventory and reports.
- `ops/env/`: overlays for `base`, `dev`, `ci`, `prod`.
- `ops/observe/`: alert rules, dashboard definitions, telemetry wiring.
- `ops/load/`: load manifests, suites, thresholds, scenarios, and query packs.
- `ops/datasets/`: dataset lifecycle contracts and versioned fixture assets.
- `ops/e2e/`: composed verification suites and expectations.
- `ops/report/`: reporting schema, generated report artifacts, and report domain references.
- `ops/_generated/`: runtime generated outputs.
- `ops/_generated.example/`: committed generated examples only.
- Canonical directory budget: keep the top-level canonical tree intentionally small; additions require contract updates and ownership review.
- Directory budget policy: `ops/DIRECTORY_BUDGET_POLICY.md`.

## Canonical SSOT Map

Authored truth:
- `ops/inventory/pins.yaml`, `ops/inventory/pin-freeze.json`, `ops/inventory/toolchain.json`, `ops/inventory/surfaces.json`, `ops/inventory/owners.json`, `ops/inventory/drills.json`, `ops/inventory/gates.json`, `ops/inventory/contracts-map.json`
- `ops/load/suites/suites.json`, `ops/load/thresholds/*.thresholds.json`, `ops/load/scenarios/*.json`, `ops/load/load.toml`
- `ops/observe/alerts/*.yaml`, `ops/observe/slo-definitions.json`, `ops/observe/alert-catalog.json`, `ops/observe/telemetry-drills.json`
- `ops/datasets/manifest.json`, `ops/datasets/promotion-rules.json`, `ops/datasets/qc-metadata.json`, `ops/datasets/rollback-policy.json`, `ops/datasets/real-datasets.json`
- `ops/e2e/suites/suites.json`, `ops/e2e/expectations/expectations.json`, `ops/e2e/reproducibility-policy.json`, `ops/e2e/taxonomy.json`

Generated truth:
- `ops/schema/generated/schema-index.json`, `ops/schema/generated/schema-index.md`, `ops/schema/generated/compatibility-lock.json`
- `ops/stack/generated/*.json`, `ops/k8s/generated/*.json`, `ops/observe/generated/telemetry-index.json`, `ops/load/generated/*.json`, `ops/report/generated/*.json`, `ops/datasets/generated/*.json`, `ops/e2e/generated/*.json`
- `ops/_generated/**` (ephemeral runtime outputs)
- `ops/_generated.example/**` (curated committed generated evidence artifacts)
- lifecycle policy: `ops/GENERATED_LIFECYCLE.md`

## Duplicate Truth Rule

- No duplicate authored truth is allowed.
- If the same semantic data appears in two paths, exactly one path must be authored and the other must be generated from it.
- Generated copies must be explicitly marked in contract and generator policy documents.
- Parallel authored registries that represent the same contract set are forbidden.

## Fixture Policy

- Versioned fixture assets are allowed only under `ops/datasets/fixtures/**`.
- Binary fixture artifacts are allowed only under versioned `assets/` directories with lock metadata.
- Every fixture version must include lock metadata and deterministic source/query fixtures.
- No loose binary assets are allowed outside the fixture policy subtree.

## Invariants

- Command surface metadata is declared in `ops/inventory/surfaces.json`.
- Ownership metadata is declared in `ops/inventory/owners.json`.
- Policy registry source is `ops/inventory/policies/dev-atlas-policy.json`.
- No semantic domain `obs` exists; only `observe` is valid across ids, keys, commands, and paths.
- Gate requirement: forbid pattern `ops\.obs\.` and forbid `"obs"` domain keys after 2026-02-25.
- Schemas live only under `ops/schema/`.
- Generated runtime outputs are written only under `ops/_generated/`.
- Committed generated outputs are written only under `ops/_generated.example/`.
- Runtime evidence and artifacts are written under `artifacts/`.
- Canonical artifact layout is defined in `ops/ARTIFACTS.md` and must not use `ops/_artifacts/`.
- `_generated.example/` accepts only curated evidence indexes and curated report examples.
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

## Contract Promotion Lifecycle

- Draft changes: update authored contracts and schemas in the same change set.
- Promotion gate: contract changes must pass `ops validate` and contract coverage checks.
- Release promotion: promoted contracts require regenerated evidence artifacts in `_generated.example`.

## Contract Deprecation

- Deprecated contract fields or files must be documented in `ops/DRIFT.md` with a removal date.
- Deprecated authored paths must retain compatibility checks until removal.
- Removal is allowed only after replacement contract paths are authoritative and enforced.

## Naming Policy

- Names use durable intent-focused nouns.
- Temporary or timeline-oriented naming is forbidden for committed paths and metadata keys.
