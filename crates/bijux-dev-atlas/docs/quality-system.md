# Quality System

Canonical testing and benchmark guidance for `bijux-dev-atlas`.

## Test Lanes

- Fast crate tests: `cargo test -p bijux-dev-atlas`
- Workspace tests: `cargo test --workspace`
- Preferred repo lane: `make test`

## Benchmark Lanes

- Compile benches: `cargo bench -p bijux-dev-atlas --no-run`
- Run benches: `cargo bench -p bijux-dev-atlas`

## Determinism Rules

- Tests and benches must remain deterministic.
- Artifacts must stay within approved workspace roots.
- Bench inputs must not mutate shared repository state.

## Related

- `testing.md` (compatibility alias)
- `benchmarks.md` (compatibility alias)
