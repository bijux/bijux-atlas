# Effect Boundary Map

## Pure Planner Surface
- `planner/mod.rs`
- `filters.rs`
- `cost.rs`
- `limits.rs`

These modules are pure and must not pull DB/network/process APIs.

## Effectful Surface
- `db/mod.rs`
- `row_decode.rs`
- `lib.rs` (query execution bridge)

## Guardrail
- `tests/purity_boundaries.rs` and `scripts/effects-lint.sh` enforce purity boundaries.
