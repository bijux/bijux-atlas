# Tool Config Shims

- Owner: `docs-governance`
- Stability: `stable`

## What

Catalog of root-level tool config symlinks and their canonical config location.

## Why

Some tools only discover config at repository root; shims preserve compatibility while keeping SSOT in `configs/`.

## Scope

Root config symlinks for rust, security, docs, and test tooling.

## Non-goals

Does not allow arbitrary new root symlinks.

## Contracts

- `clippy.toml` -> `configs/rust/clippy.toml`
- `rustfmt.toml` -> `configs/rust/rustfmt.toml`
- `nextest.toml` -> `configs/nextest/nextest.toml`
- `deny.toml` -> `configs/security/deny.toml`
- `audit-allowlist.toml` -> `configs/security/audit-allowlist.toml`
- `.vale.ini` -> `configs/docs/.vale.ini`
- `.vale` -> `configs/docs/.vale`

## Failure modes

- Direct root config edits bypass canonical config ownership.
- Undocumented shims create non-reproducible local behavior.

## How to verify

```bash
$ make layout-check
```

Expected output: symlink policy checks pass.

## See also

- [Symlink Index](symlinks.md)
- [Repository Surface](repo-surface.md)
- `configs/README.md`
