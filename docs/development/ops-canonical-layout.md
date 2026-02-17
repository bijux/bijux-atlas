# Ops Canonical Layout Policy

- Owner: `docs-governance`

## What

Defines the final repository policy for operational assets.

## Why

Keeps operational artifacts in one stable location and prevents root-level sprawl.

## Contracts

- `ops/` is the only canonical home for:
  - `e2e`
  - `load`
  - `observability`
  - `openapi`
- Legacy root aliases are forbidden (`charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`).
- Root `charts/` is packaging-only.
  - Helm packaging uses `ops/k8s/charts/bijux-atlas/`.
  - Operational validation and tests run from `ops/` workflows.
- `make` is the supported runnable interface.
  - Scripts are internal implementation details.
- Operational outputs must be written under `artifacts/ops/<run-id>/`.
- Root tool configs are allowed only as symlink shims when tools require root discovery.
- Compatibility symlinks must be documented and minimal.
  - Current list: `nextest.toml`, `deny.toml`, `audit-allowlist.toml`, `clippy.toml`, `rustfmt.toml`, `.vale.ini`, `.vale`, `ops/tool-versions.json`, `datasets`, `fixtures`.
- `target/` and `.DS_Store` are never committed.
- `.idea/` is ignored and never committed.

## Failure modes

Root drift creates duplicate truth and unstable ops behavior.

## How to verify

```bash
$ make layout-check
$ make no-direct-scripts
```

Expected output: both checks pass with no drift warnings.

## See also

- [Repository Surface](repo-surface.md)
- [Makefiles Surface](makefiles/surface.md)
- [Ops Layout](../../ops/README.md)
