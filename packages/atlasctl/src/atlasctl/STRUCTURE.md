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

## Migration Notes

- Check implementations are consolidating under `checks/<domain>/...`.
- Canonical checks command module is `commands/check/command.py`; `checks/command.py` is compatibility-only.
- Old deep paths under `checks/layout/.../checks/` are compatibility shims; canonical implementations live under:
  - `checks/make/impl/`
  - `checks/ops/impl/`
- Repo native check split modules are consolidated under:
  - `checks/repo/native/modules/`
  - `checks/repo/native/runtime_modules/`
  with `checks/repo/native_loader.py` and `checks/repo/native_runtime.py` kept as compatibility shims.
- Core execution/model canonical homes:
  - `core/effects/exec.py` and `core/effects/exec_shell.py` (with `core/exec*.py` compatibility shims)
  - `core/model/` (with `core/models/` compatibility package)
