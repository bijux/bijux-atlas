# Lint Policy

This repository enforces a strict lint baseline with explicit scope:

- `unsafe` is forbidden.
- `unwrap`/`expect` are denied in production code.
- `unwrap`/`expect` are allowed in tests only.
- `todo!()` is forbidden everywhere.
- `dbg!()`, `println!()`, and `eprintln!()` are denied by clippy policy.
- `unused_crate_dependencies` is denied.

Source of truth:

- Workspace lint levels: `Cargo.toml` under `[workspace.lints.*]`
- Clippy test allowances: `configs/rust/clippy.toml`

CI enforcement:

- `make lint` runs `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `make lint-policy-report` writes `artifacts/lint/effective-clippy-policy.txt`
- `make lint-policy-enforce` enforces drift checks for `todo!`, `dbg!`, non-test `println!/eprintln!` in core crates, and `reqwest/blocking` in server
- `make lint-clippy-json` writes `artifacts/lint/clippy.json`
