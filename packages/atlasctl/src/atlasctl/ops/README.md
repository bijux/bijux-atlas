# atlasctl.ops subsystem

The ops subsystem is the Python-native execution surface for infrastructure, deployment, observability, load, and E2E operations.

## Execution model (one page)

- Public CLI enters via `atlasctl commands/ops/command.py`.
- Area commands (`commands/ops/<area>/command.py`) are thin dispatch only.
- Runtime orchestration lives in `commands/ops/<area>/runtime.py` or `commands/ops/runtime_modules/*`.
- External tools are invoked through adapters in `commands/ops/tools.py` and `core.process`.
- Reports/evidence are schema-validated and written under `artifacts/evidence/<area>/<run_id>/...`.

## Boundary rules

- No direct `ops/run/*.sh` wrappers.
- No imports from `checks/layout/*` or policy internals.
- No imports from test/fixture modules in command code.
- Commands use `RunContext` (and optional narrowed `CommandContext`) for `run_id`,
  `repo_root`, `write_roots`, and profile selection.
- Global `--profile` and `--allow-network`/`--network` flags are the canonical
  profile/network controls for ops actions.
- Ops action capabilities (tools/network/profile expectations) are declared in
  `configs/ops/command-capabilities.json`.
