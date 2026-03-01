# BENCHMARKS (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## Purpose

Reference for benchmark execution and isolation expectations for the dev control-plane crate.

## Bench Commands

- Compile all benches (fast verification):
  - `cargo bench -p bijux-dev-atlas --no-run`
- Compile one bench:
  - `cargo bench -p bijux-dev-atlas --bench core_engine --no-run`
- Run benches (local measurement):
  - `cargo bench -p bijux-dev-atlas`

## Isolation Rules

- Bench output and build artifacts must stay under the workspace target/artifact isolation roots.
- Bench groups must have unique `criterion_group!` names per bench file.
- Bench inputs/outputs must be deterministic and must not mutate shared repository state.

## Existing Benches

- `core_engine`
- `file_walk`
- `inventory_scan`
- `policy_eval`
- `report_codec`

## Related

- Architecture invariants: `crates/bijux-dev-atlas/docs/architecture.md`
- Test lanes and nextest usage: `crates/bijux-dev-atlas/docs/testing.md`
