# Rust Toolchain Contract

The Rust toolchain contract for this repository is rooted at the repo root.

## Canonical Files

- `rust-toolchain.toml`: pinned Rust channel and required components
- `.cargo/config.toml`: deterministic Cargo build defaults and target directory isolation
- `rustfmt.toml`: formatter policy
- `clippy.toml`: lint policy
- `configs/nextest/nextest.toml`: nextest execution profile and isolated store path
- `configs/security/deny.toml`: cargo-deny policy
- `configs/security/audit-allowlist.toml`: audited exception allowlist

## CI Contract

CI must install the exact toolchain pinned in `rust-toolchain.toml`.
The Rust workflow must not use a floating toolchain label for the dev control-plane lane.

## Makefile Contract

Makefile cargo gates may call `cargo` directly for developer ergonomics, but control-plane governance and repo policy checks remain under `bijux dev atlas ...`.
