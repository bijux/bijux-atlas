# Cargo Execution Contract

All build/test/lint/audit commands must run inside an isolate runner.

## Required Runner

- `cargo build/test wrappers use isolated artifacts paths`: creates isolated runtime rooted at `artifacts/isolate/<tag>`.
- `artifact isolation is enforced by dev-atlas wrappers`: validates the isolation contract.

## Environment Contract

Required vars:

- `ISO_TAG`
- `ISO_RUN_ID`
- `ISO_ROOT`
- `CARGO_TARGET_DIR`
- `CARGO_HOME`
- `TMPDIR`
- `TMP`
- `TEMP`

Required invariants:

- `ISO_ROOT` must be under `artifacts/isolate/`.
- `CARGO_TARGET_DIR`, `CARGO_HOME`, `TMPDIR`, `TMP`, `TEMP` must all be under `ISO_ROOT`.

## Test Runner Policy

- `nextest` is the default test runner (`make test`).
- `make test-all` runs all tests including ignored tests (no skips).
- Deterministic scheduling is configured in `configs/nextest/nextest.toml`.

## Make Targets

- `make fmt`
- `make lint`
- `make check`
- `make test`
- `make test-all`
- `make ci-deny`
- `make audit`
- `make ci-license-check`
- `make policy-lint`
- `make docs`
- `make openapi-drift`
