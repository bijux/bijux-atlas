# Root Inventory

- Owner: `docs-governance`

## What

Defines the approved root-level repository surface and why each item remains.

## Why

Prevents root sprawl and keeps navigation and automation predictable.

## Root Entries

- `.cargo/`: Cargo root-discovery workspace config.
- `.github/`: CI workflows.
- `artifacts/`: isolated and ops outputs.
- `bin/`: compatibility command shims that redirect to `atlasctl`.
- `configs/`: canonical static configuration files.
- `crates/`: Rust workspace crates.
- `docker/`: canonical Docker build surface.
- `docs/`: documentation site source.
- `makefiles/`: make target implementations.
- `ops/`: canonical operations assets.
- legacy script-tree paths: internal automation shims.
- `packages/atlasctl/`: code generation and automation CLI package.

## Failure modes

Unexpected root additions create undocumented interfaces and drift.

## How to verify

```bash
$ make atlasctl-check-layout
```

Expected output: root-shape check passes.

## See also

- [Repository Surface](repo-surface.md)
- [Symlink Index](symlinks.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
