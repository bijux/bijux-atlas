# Contract Compatibility

This crate owns persisted model contracts consumed across atlas components.

## Contract Artifacts

- `ArtifactManifest`
- `Catalog`
- `ShardCatalog`
- `ReleaseGeneIndex`
- `DiffPage`
- `IngestAnomalyReport`

## Compatibility Promise

- New fields are additive and must include safe defaults for older artifacts.
- `ModelVersion` exists on persisted top-level artifacts and defaults to `v1` when absent.
- Field names remain stable unless accompanied by explicit compatibility notes.
- `#[serde(deny_unknown_fields)]` is used on strict contracts where rejecting unexpected input is safer.

## Validation Boundary

Each top-level contract must provide pure `validate()` checks with no I/O.

## Fixture Policy

- `tests/fixtures/current/` stores canonical fixtures for current contracts.
- `tests/fixtures/v0_1/` stores backward-compatibility fixtures parsed in CI.
