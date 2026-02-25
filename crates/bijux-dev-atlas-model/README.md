# bijux-dev-atlas-model

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Model and serialization contract crate for dev control-plane reports.

## Purpose
- Define typed report schemas shared by engine, CLI, and downstream tooling.
- Keep machine-stable identifiers (`CheckId`, `ViolationId`, `ArtifactPath`) explicit.
- Provide deterministic fingerprint helpers for violation ratchets.

## Update Workflow
1. Change model types in `src/lib.rs`.
2. Keep `schema_version` behavior explicit for persisted artifacts.
3. Update tests in `tests/serde_roundtrip.rs` and `tests/golden_fixture.rs`.
4. Update fixtures in `tests/fixtures/` if contract shape changes.
5. Run `cargo test -p bijux-dev-atlas-model`.

## Stability
- Serialized report contracts are stable by default.
- Breaking shape changes require schema version review.
