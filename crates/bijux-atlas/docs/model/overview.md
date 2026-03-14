# bijux-atlas-model

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Data model crate for persisted atlas contracts and serde-compatible domain types.

## Contract Artifacts Owned

- `ArtifactManifest`
- `Catalog`
- `ShardCatalog`
- `ReleaseGeneIndex`
- `DiffPage`
- `IngestAnomalyReport`

## Stability and Compatibility Rules

- Public model contracts evolve additively.
- Top-level persisted artifacts include `ModelVersion` (defaulting to `v1` for older payloads).
- Unknown fields are denied on strict contract structs where forward ambiguity is unsafe.
- Contract changes require fixture updates in `tests/fixtures/current` and compatibility checks in `tests/fixtures/v0_1`.

## Validation Boundary

Top-level models provide pure `validate()` methods. They must not perform I/O.

## Benchmarks

- `model_codec`: encode/decode throughput for large manifest payloads.
- parsing benches for key identifiers and region parsing.

## Docs

- `docs/contract-compatibility.md`
- `docs/schema-stability.md`
- `docs/public-api.md`

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
