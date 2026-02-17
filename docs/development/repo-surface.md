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
- Policy/config roots: `configs/`, `docs/contracts/*.json`

## Failure modes

Surface drift breaks automation and team workflows.

## How to verify

```bash
$ python3 scripts/docs/check_make_targets_documented.py
$ python3 scripts/docs/check_script_headers.py
```

Expected output: both checks pass.

## See also

- [Development Index](INDEX.md)
- [Scripts Index](scripts/INDEX.md)
- [Contracts Index](../contracts/INDEX.md)
