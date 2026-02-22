# Tool Config Shims

- Owner: `docs-governance`
- Stability: `stable`

## What

Historical note for removed root tool-config symlinks and current explicit-path usage.

## Why

Tooling now uses explicit config paths from `configs/` or `.cargo/config.toml` defaults where applicable.

## Scope

Rust, security, and docs tool configurations under `configs/`.

## Non-goals

Does not allow root tool-config symlinks.

## Contracts

- `cargo fmt` uses `--config-path configs/rust/rustfmt.toml`.
- `cargo clippy` uses `CLIPPY_CONF_DIR=configs/rust`.
- `cargo deny` uses `--config configs/security/deny.toml`.
- `vale` uses `--config=configs/docs/.vale.ini`.

## Failure modes

- Root config files can drift from canonical SSOT.
- Implicit root discovery hides config provenance.

## How to verify

```bash
$ make atlasctl-check-layout
```

Expected output: symlink policy checks pass.

## See also

- [Symlink Index](symlinks.md)
- [Repository Surface](repo-surface.md)
- `configs/README.md`
