# CONTRACT (bijux-atlas-bench)

- Owner: bijux-atlas-performance
- Stability: stable

## Inputs

- Benchmark configuration values and dataset registry entries.
- Harness metric samples from deterministic benchmark runs.

## Outputs

- Stable benchmark metadata models and validators.
- Deterministic fixture registry for repeatable benchmark setup.

## Invariants

- Validation rules stay deterministic.
- Dataset tiers remain explicit and reviewable.
- Result schema fields remain stable across runs.

## Anti-patterns

- Runtime-specific benchmark tuning logic inside shared data models.
- Non-deterministic defaults derived from wall-clock or host-specific state.
- Silent schema drift without contract and docs updates.

## Dependencies allowed

- Rust standard library.
- Workspace crates required for benchmark model serialization and validation.

## Effects policy

- This crate is pure and must not perform filesystem, network, process, or clock effects.

## Error policy

- Validation errors must be returned with stable, typed error variants.
- Error messages must remain deterministic for identical inputs.

## Versioning/stability

- Public structs and enums follow additive-compatible evolution by default.
- Breaking model changes require explicit contract and changelog updates.

## Tests expectations

- Unit tests cover config parsing, dataset tier validation, and result model invariants.
- Test fixtures remain deterministic and do not depend on host environment state.

## Public API surface

- `config`, `dataset`, and `harness` modules expose stable benchmark model primitives.
