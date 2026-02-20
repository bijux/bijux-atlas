# Symlink Index

- Owner: `docs-governance`

## What

Lists all repository-level compatibility symlinks and their rationale.

## Why

Prevents hidden compatibility behavior and enforces explicit symlink governance.

## Symlinks

Policy rule:
- Only compatibility shims are allowed at root.
- Allowed shim class: `Dockerfile` shim.
- New symlinks require:
  - An entry in `configs/repo/symlink-allowlist.json`.
  - A `docs/development/symlinks.md` entry with `APPROVAL-*` token.

- `Dockerfile` -> `docker/images/runtime/Dockerfile`: root compatibility for tooling expecting root Dockerfile. (Approval: `APPROVAL-DOCKERFILE-SHIM`)

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
