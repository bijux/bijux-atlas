# Symlink Index

- Owner: `docs-governance`

## What

Lists all repository-level compatibility symlinks and their rationale.

## Why

Prevents hidden compatibility behavior and enforces explicit symlink governance.

## Symlinks

Policy rule:
- Only compatibility shims are allowed at root.
- Allowed shim classes: tool-config discovery shims, `Dockerfile` shim, and `bin` UX shim.
- New symlinks require a `docs/development/symlinks.md` entry with `APPROVAL-*` token.

- `Dockerfile` -> `docker/Dockerfile`: root compatibility for tooling expecting root Dockerfile. (Approval: `APPROVAL-DOCKERFILE-SHIM`)
- `bin` -> `scripts/bin`: root compatibility while `scripts/bin` is canonical. (Approval: `APPROVAL-SCRIPT-BIN-SHIM`)
- `deny.toml` -> `configs/security/deny.toml`: tool root-discovery compatibility. (Approval: `APPROVAL-DENY-SHIM`)
- `audit-allowlist.toml` -> `configs/security/audit-allowlist.toml`: tool root-discovery compatibility. (Approval: `APPROVAL-AUDIT-ALLOWLIST-SHIM`)
- `clippy.toml` -> `configs/rust/clippy.toml`: tool root-discovery compatibility. (Approval: `APPROVAL-CLIPPY-SHIM`)
- `rustfmt.toml` -> `configs/rust/rustfmt.toml`: tool root-discovery compatibility. (Approval: `APPROVAL-RUSTFMT-SHIM`)
- `.vale` -> `configs/docs/.vale`: tool root-discovery compatibility. (Approval: `APPROVAL-VALE-DIR-SHIM`)
- `.vale.ini` -> `configs/docs/.vale.ini`: tool root-discovery compatibility. (Approval: `APPROVAL-VALE-INI-SHIM`)

Non-root compatibility pointer:
- `ops/e2e/stack` -> `ops/stack`: compatibility pointer from e2e harness to canonical stack manifests.

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
- [Tool Config Shims](tool-config-shims.md)
- [Makefiles Surface](makefiles/surface.md)
