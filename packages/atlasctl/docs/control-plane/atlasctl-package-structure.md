# Atlasctl Structure Contract

This file declares the intended top-level module layout for `packages/atlasctl/src/atlasctl`.

## Intended Top-Level Packages (max 10)

- `cli/`
- `commands/`
- `checks/`
- `core/`
- `contracts/`
- `registry/`
- `suite/`
- `reporting/`

`policies` and `internal` are command groups (under `commands/`), not required top-level python packages.

## Checks Layout Notes

- Canonical checks runtime surfaces are:
  - `checks/model.py`
  - `checks/registry.py`
  - `checks/selectors.py`
  - `checks/policy.py`
  - `checks/runner.py`
  - `checks/report.py`
- Check definitions are exported from flat `checks/domains/*.py` modules.
- Shared check helpers live under `checks/tools/`.
- Legacy compatibility trees under `checks/layout/` and `checks/repo/` are blocked by internal checks and are being removed.
