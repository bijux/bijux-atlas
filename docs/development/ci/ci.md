# CI Surface

- Owner: `build-and-release`

## Canonical Entry Points

- `make ci` -> `./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci`
- `make ci-fast` -> `./bin/atlasctl ci fast --json`
- `make ci-nightly` -> `./bin/atlasctl ci nightly --json`
- `make ci-help` -> `./bin/atlasctl help ci`

## Workflow Rules

- CI workflows must call only `make <approved-target>` or `./bin/atlasctl ...`.
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
  - `./bin/atlasctl report ci-summary --latest`
- CI lane registry:
  - `./bin/atlasctl ci list --json`
