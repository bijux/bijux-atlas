# Rust toolchain configs

- Owner: `platform`
- Purpose: define Rust formatting, linting, and toolchain version inputs.
- Consumers: rustfmt, clippy, cargo workflows, and Rust CI lanes.
- Update workflow: update toolchain inputs with the matching Rust lane change, then rerun format and lint checks.
