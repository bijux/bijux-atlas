# Atlasctl Architecture North Star

## What atlasctl is (and is not)

- `atlasctl` is the single internal tooling CLI for this repository.
- `atlasctl` owns command orchestration, policy checks, and contract/report generation.
- `atlasctl` uses `argparse` as the command parser (no `typer` migration planned now).
- `atlasctl` command modules are wiring + orchestration only, not ad-hoc script buckets.
- `atlasctl` commands invoke process effects through `atlasctl.core.exec` only.
- `atlasctl` commands write files through `atlasctl.core.fs` with allowed-root policy only.
- `atlasctl` canonical top-level package map is: `core`, `cli`, `commands`, `checks`, `contracts`.
- `atlasctl` command module naming convention is `command.py` per domain package.
- `atlasctl` uses `checks/` as the canonical check domain; `check/` is legacy compatibility only.
- `atlasctl` is not the runtime app, not a user SDK, and not a replacement for crate/runtime logic.

## Boundaries And Domain Rules

- Domain commands are stable user-facing top-level domains such as `registry` and `layout` that own a bounded command family.
- Normal commands are leaf actions under a domain and do not introduce new domain boundaries.
- Makefiles may expose convenience targets but must delegate behavior to `atlasctl` as source of truth.

See `docs/atlasctl/BOUNDARIES.md` for effect boundaries, invariants, output contracts, and policy enforcement rules.

## Budget Policy

- Budget SSOT is `packages/atlasctl/pyproject.toml` under `[tool.atlasctl.budgets]`.
- Canonical keys include:
  - `max_py_files_per_dir`
  - `max_modules_per_dir`
  - `max_loc_per_file`
  - `max_loc_per_dir`
- Subtree-specific budget overrides are defined with `[[tool.atlasctl.budgets.rules]]`.

### Budget Exceptions

- `packages/atlasctl/src/atlasctl/legacy/docs_runtime_chunks`: legacy compatibility runtime shards with dynamic loading.
