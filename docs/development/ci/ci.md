# CI Surface

- Owner: `build-and-release`

## Canonical Entry Points

- `make ci` -> `bijux dev atlas ci run --json --out-dir artifacts/reports/dev-atlas/suite-ci`
- `make ci-fast` -> `bijux dev atlas ci fast --json`
- `make ci-nightly` -> `bijux dev atlas ci nightly --json`
- `make ci-help` -> `bijux dev atlas help ci`

## Workflow Rules

- CI workflows must call only `make <approved-target>` or `bijux dev atlas ...`.
- Direct `cargo test/fmt/clippy` or `pytest` in workflows is forbidden unless explicitly allowlisted.
- Workflows running CI must upload report/log artifacts with `if: always()`.

## Determinism

- CI runs write evidence under `artifacts/evidence/ci/<run_id>/`.
- CI defaults to isolation mode; deterministic outputs are required under controlled env.

## Cache Policy

- Cache keys must include lockfiles and toolchain versions.
- Minimum cache key material:
  - `Cargo.lock`
  - rust toolchain version
  - python version (when python environment is cached)

## Reporting

- Unified CI summary command:
  - `bijux dev atlas report ci-summary --latest`
- CI lane registry:
  - `bijux dev atlas ci list --json`
