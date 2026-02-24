# bijux-atlas-model

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

- `docs/CONTRACT_COMPATIBILITY.md`
- `docs/SCHEMA_STABILITY.md`
- `docs/public-api.md`
