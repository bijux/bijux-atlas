# CI Surface

- Owner: `build-and-release`

## Canonical Entry Points

- `make ci` -> `./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci`
- `make ci-fast` -> `./bin/atlasctl ci fast --json`
- `make ci-all` -> `./bin/atlasctl ci all --json`
- `make ci-contracts` -> `./bin/atlasctl ci contracts --json`
- `make ci-docs` -> `./bin/atlasctl ci docs --json`
- `make ci-ops` -> `./bin/atlasctl ci ops --json`
- `make ci-release` -> `./bin/atlasctl ci release --json`
- `make ci-release-all` -> `./bin/atlasctl ci release-all --json`
- `make ci-init` -> `./bin/atlasctl ci init --json`
- `make ci-artifacts` -> `./bin/atlasctl ci artifacts --json`
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
