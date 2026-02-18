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
  - Allowlist source: `scripts/layout/root_whitelist.json`.
  - Classification is explicit and enforced:
    - `required`: must exist at root.
    - `allowed`: allowed at root.
    - `compat_shims`: root compatibility symlinks only.
    - `local_noise`: allowed locally, ignored by CI cleanliness gates unless tracked.
  - `ops/` is the canonical home for `e2e`, `load`, `observability`, and `openapi`.
  - `configs/` holds policy, rust, docs, and security config sources.
  - `configs/README.md` is the configuration layout contract.
  - Root config files are compatibility symlinks to `configs/*`.
  - Allowed root shims are limited to tool config shims plus `Dockerfile` and `bin`.
  - Legacy root aliases (`charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`) are forbidden.
  - Root `charts/` is packaging-only; ops execution and tests run from `ops/`.
  - `.cargo/` remains at root because Cargo workspace config discovery expects it.
  - Operational results belong under `artifacts/ops/<run-id>/`.
  - `.idea/` is ignored; `target/` and `.DS_Store` are never committed.
- Single entrypoint policy:
  - All runnable workflows are exposed through `make`.
  - CI workflows must not run scripts directly; `make no-direct-scripts` is the enforcement gate.

## Failure modes

Surface drift breaks automation and team workflows.

## How to verify

```bash
$ python3 scripts/docs/check_make_targets_documented.py
$ python3 scripts/docs/check_script_headers.py
$ make layout-check
$ make no-direct-scripts
```

Expected output: all checks pass.

## See also

- [Development Index](INDEX.md)
- [Repository Layout](repo-layout.md)
- [Scripts Index](scripts/INDEX.md)
- [Contracts Index](../contracts/INDEX.md)
