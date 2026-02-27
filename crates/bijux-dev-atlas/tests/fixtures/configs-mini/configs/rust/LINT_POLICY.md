# Rust Lint Policy

This document mirrors the enforced CI lint commands.

## Commands

- `cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml`
- `CLIPPY_CONF_DIR=configs/rust cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --locked`

## Scope

- `unwrap` and `expect` are allowed only in tests.
- `todo!`, `dbg!`, and non-test `println!/eprintln!` are forbidden by repository checks.
