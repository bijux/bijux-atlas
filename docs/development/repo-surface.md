# Repository Surface

- Owner: `docs-governance`

## What

Defines stable repository-level interfaces.

## Why

Prevents accidental breakage of operator/developer workflows.

## Scope

Make targets, script interfaces, and contract/config files.

## Non-goals

Does not freeze internal implementation details.

## Contracts

- Stable make targets: [`makefiles/surface.md`](makefiles/surface.md)
- Script interfaces: [`scripts/INDEX.md`](scripts/INDEX.md)
- SSOT contracts: [`../contracts/INDEX.md`](../contracts/INDEX.md)
- Root layout SSOT:
  - `ops/` holds `e2e`, `load`, `observability`, and `openapi`.
  - `configs/` holds policy, rust, docs, and security config sources.
  - Root config files are compatibility symlinks to `configs/*`.
  - `.cargo/` remains at root because Cargo workspace config discovery expects it.
- Single entrypoint policy:
  - All runnable workflows are exposed through `make`.
  - Direct script execution is diagnostic-only unless explicitly documented.

## Failure modes

Surface drift breaks automation and team workflows.

## How to verify

```bash
$ python3 scripts/docs/check_make_targets_documented.py
$ python3 scripts/docs/check_script_headers.py
$ ./scripts/layout/check_root_shape.sh
```

Expected output: all checks pass.

## See also

- [Development Index](INDEX.md)
- [Scripts Index](scripts/INDEX.md)
- [Contracts Index](../contracts/INDEX.md)
