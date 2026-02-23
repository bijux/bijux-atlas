# Atlasctl Shim Delete Plan

Cutoff target: `2026-04-01` (cleanup PRs must delete or replace all entries below).

## Legacy Shims / Facades

- `packages/atlasctl/src/atlasctl/core/context.py`
  - Replacement: `packages/atlasctl/src/atlasctl/runtime/context.py`
- `packages/atlasctl/src/atlasctl/core/exec_shell.py`
  - Replacement: `packages/atlasctl/src/atlasctl/core/exec.py`
- `packages/atlasctl/src/atlasctl/core/effects/exec_shell.py`
  - Replacement: `packages/atlasctl/src/atlasctl/core/exec_shell.py`
- `packages/atlasctl/src/atlasctl/core/effects/network.py`
  - Replacement: `packages/atlasctl/src/atlasctl/core/network.py`
- `packages/atlasctl/src/atlasctl/checks/core/execution.py`
  - Replacement: `packages/atlasctl/src/atlasctl/checks/engine/execution.py`
- `packages/atlasctl/src/atlasctl/cli/main.py` (`main()` only)
  - Replacement: `packages/atlasctl/src/atlasctl/app/main.py`

## Removal PR Order

1. Migrate imports to canonical modules (`runtime/*`, `checks/engine/execution`).
2. Turn shims into fail-fast stubs with migration messages.
3. Delete shim modules and update import-boundary allowlists/checks.
4. Regenerate checks registry/docs metadata and run repo native checks.

## Scope Note

`ops` remains repository spec; `atlasctl` remains behavior implementation. This plan deletes compatibility shims only and does not change command surface contracts.
