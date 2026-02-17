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
- `bin` (symlink): compatibility alias to `scripts/bin`.
- `charts` (symlink): compatibility alias to `ops/k8s/charts`.
- `configs/`: canonical static configuration files.
- `crates/`: Rust workspace crates.
- `datasets` (symlink): compatibility alias to `ops/datasets`.
- `docker/`: canonical Docker build surface.
- `docs/`: documentation site source.
- `e2e` (symlink): compatibility alias to `ops/e2e`.
- `fixtures` (symlink): compatibility alias to `ops/fixtures`.
- `load` (symlink): compatibility alias to `ops/load`.
- `makefiles/`: make target implementations.
- `observability` (symlink): compatibility alias to `ops/observability`.
- `ops/`: canonical operations assets.
- `scripts/`: internal automation scripts.
- `xtask/`: code generation and automation crate.

## Failure modes

Unexpected root additions create undocumented interfaces and drift.

## How to verify

```bash
$ make layout-check
```

Expected output: root-shape check passes.

## See also

- [Repository Surface](repo-surface.md)
- [Symlink Index](symlinks.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
