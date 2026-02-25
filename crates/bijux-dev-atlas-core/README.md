# bijux-dev-atlas-core

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Deterministic engine crate for `bijux dev atlas check ...` execution.

## Stability
- Check ids and report JSON shape are treated as stable contracts.
- Registry and suite ordering are deterministic and validated.
- Effects are explicit and capability-gated in the runner.

## Add A Check
1. Add a new check entry in `ops/inventory/registry.toml` with stable `id`.
2. Assign domain, tags, suites, and required effects.
3. Set a non-zero execution budget.
4. Implement check logic in `src/checks/`.
5. Add the implementation id mapping in `src/checks/ops.rs`.
6. Ensure the check reads/writes only through provided adapters.
7. Return `Violation` records with machine-stable codes.
8. Add unit or integration coverage for pass/fail behavior.
9. Run `cargo test -p bijux-dev-atlas-core`.
10. Run registry doctor and keep output deterministic.

## Development Commands
- Test: `cargo test -p bijux-dev-atlas-core`
- Bench: `cargo bench -p bijux-dev-atlas-core --bench core_engine`

## Compatibility Rules
- Do not rename or recycle existing check ids.
- Do not add timestamp-dependent paths to evidence.
- Do not bypass runner capability gating for effects.
