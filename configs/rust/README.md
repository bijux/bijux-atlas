# Rust Tooling Config

Canonical Rust tool configuration lives here.

- `clippy.toml`: lint policy used by `cargo clippy`.
- `rustfmt.toml`: formatting policy used by `cargo fmt`.

Root files `clippy.toml` and `rustfmt.toml` are symlinks only.

## Verification

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
