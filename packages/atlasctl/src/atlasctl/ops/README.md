# atlasctl.ops subsystem

The ops subsystem is the Python-native execution surface for infrastructure, deployment, observability, load, and E2E operations.

## Execution model (one page)

- Public CLI enters via `atlasctl commands/ops/command.py`.
- Area commands (`commands/ops/<area>/command.py`) are thin dispatch only.
- First-class ops workflows live under `atlasctl/ops/workflows/` and are the preferred orchestration layer.
- Tool invocations are wrapped by `atlasctl/ops/adapters/` (`kubectl`, `helm`, `kind`, `docker`, `k6`).
- Shared structured payloads live in `atlasctl/ops/models/` (toolchain, topology, reports).
- Runtime orchestration lives in `commands/ops/<area>/runtime.py` or `commands/ops/runtime_modules/*`.
- External tools are invoked through adapters in `commands/ops/tools.py` and `core.process`.
- Reports/evidence are schema-validated and written under `artifacts/evidence/<area>/<run_id>/...`.

## Boundary rules

- No direct `ops/run/*.sh` wrappers.
- Embedded shell assets under `commands/ops/runtime_modules/assets/` are LEGACY migration shims; new workflows should be Python orchestration in `atlasctl.ops.workflows`.
- No imports from `checks/layout/*` or policy internals.
- No imports from test/fixture modules in command code.
- `obs` is a public facade; `observability` owns observability business logic.
- Commands use `RunContext` (and optional narrowed `CommandContext`) for `run_id`,
  `repo_root`, `write_roots`, and profile selection.
- Global `--profile` and `--allow-network`/`--network` flags are the canonical
  profile/network controls for ops actions.
- Ops action capabilities (tools/network/profile expectations) are declared in
  `configs/ops/command-capabilities.json`.
