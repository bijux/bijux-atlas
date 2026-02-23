# atlasctl Package Architecture

This document describes the package-level layering used by `packages/atlasctl`.

## Layer Model

1. `cli/`
: argument parsing + public command surface registry only.

2. `commands/`
: command group entrypoints (`command.py`) and thin dispatch into runtime/domain modules.

3. `commands/*/runtime.py` and `commands/ops/runtime_modules/*`
: command-domain runtime orchestration and policy-aware action wiring.

4. `core/`
: shared process/effects/context/fs/schema/runtime adapters.

5. `contracts/` + `contracts/schema/`
: schemas and validation contracts for outputs/manifests.

6. `registry/`
: canonical local catalogs (checks/suites/ops tasks) and typed readers.

## Dependency Direction

- `cli -> commands -> runtime/domain -> core/contracts/registry`
- `commands` must not depend on checks/layout/policies internals.
- `commands/ops` must not import test or fixture modules.

## Command Group Rule

Each public command group exposes one public entry module (`commands/<group>/command.py`).
Business logic belongs in `runtime.py` or area/runtime modules.

## Result + Output Rule

- Command/runtime functions should return structured status/results where practical and
  standardize result typing via `atlasctl.core.result` (not implementation-path imports).
- Ops command entrypoints must not print directly; they should emit through a shared
  output adapter (`commands/ops/_shared/output.py`) so output policy is enforceable.

## Ops Runtime Execution Rules

- Commands resolve execution state from `RunContext`, and may project it into a
  narrower `CommandContext` (`core/runtime/command_context.py`) for runtime modules.
- Path resolution must go through explicit repo-root-relative paths (`ctx.repo_root / ...`)
  and shared path helpers in `core/runtime/paths.py` (no `Path('.')` reliance).
- Networked actions require explicit operator intent (`--allow-network` or
  `--network=allow`) and are gated by command-group policy + ops tool preflight.
- Ops action capabilities are declared in `configs/ops/command-capabilities.json`.
