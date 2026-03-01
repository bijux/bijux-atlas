# TESTING (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## Purpose

Reference for running dev-atlas tests and interpreting the main test lanes.

## Common Commands

- Fast crate test pass:
  - `cargo test -p bijux-dev-atlas`
- Full workspace test pass:
  - `cargo test --workspace`
- Preferred repo lane (nextest):
  - `make test`
- Slow-only tests (when diagnosing contract drift):
  - `make test-slow`

## Key Test Groups (dev-atlas)

- `tests/effects_boundary.rs`: host-effect boundary grep gates
- `tests/boundaries.rs`: module dependency boundary checks
- `tests/cli_*`: command surface and contract snapshots
- `tests/governance_*`: registry/taxonomy/module-budget governance checks
- `tests/policies_*`: dev policy schema/validation/ratchet/expiry behavior

## Determinism and Isolation

- Tests use the workspace `artifacts/target` cache root (via workspace cargo config).
- Tests must not write outside approved artifact roots.
- Slow docs/doctor contract tests are allowed to take longer; they must remain deterministic.

## Related

- Bench commands and expectations: `crates/bijux-dev-atlas/docs/benchmarks.md`
- Behavior contract: `crates/bijux-dev-atlas/CONTRACT.md`
