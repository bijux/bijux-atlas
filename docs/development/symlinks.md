# Symlink Index

- Owner: `docs-governance`

## What

Lists all repository-level compatibility symlinks and their rationale.

## Why

Prevents hidden compatibility behavior and enforces explicit symlink governance.

## Symlinks

- `Dockerfile` -> `docker/Dockerfile`: root compatibility for tooling expecting root Dockerfile.
- `bin` -> `scripts/bin`: root compatibility while `scripts/bin` is canonical.
- `nextest.toml` -> `configs/nextest/nextest.toml`: tool root-discovery compatibility.
- `deny.toml` -> `configs/security/deny.toml`: tool root-discovery compatibility.
- `audit-allowlist.toml` -> `configs/security/audit-allowlist.toml`: tool root-discovery compatibility.
- `clippy.toml` -> `configs/rust/clippy.toml`: tool root-discovery compatibility.
- `rustfmt.toml` -> `configs/rust/rustfmt.toml`: tool root-discovery compatibility.
- `.vale` -> `configs/docs/.vale`: tool root-discovery compatibility.
- `.vale.ini` -> `configs/docs/.vale.ini`: tool root-discovery compatibility.
- `ops/tool-versions.json` -> `../configs/ops/tool-versions.json`: compatibility alias for prior path.

## Failure modes

Untracked symlinks create undocumented behavior and break reproducibility.

## How to verify

```bash
$ make layout-check
```

Expected output: symlink index check and layout checks pass.

## See also

- [Repository Surface](repo-surface.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
- [Makefiles Surface](makefiles/surface.md)
