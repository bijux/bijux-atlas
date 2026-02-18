# Rust Tooling Config

Canonical Rust tool configuration lives here.

- `clippy.toml`: lint policy used by `cargo clippy`.
- `rustfmt.toml`: formatting policy used by `cargo fmt`.
- `rust-toolchain.toml` (root): pinned toolchain selector consumed by cargo/rustup.

Root files `clippy.toml` and `rustfmt.toml` are symlinks only.

## Policy

- Clippy warnings are treated as errors in CI (`-D warnings`).
- Formatting policy is enforced via `cargo fmt --check`.
- Toolchain pin must be updated intentionally with compatibility validation.

## Verification

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
