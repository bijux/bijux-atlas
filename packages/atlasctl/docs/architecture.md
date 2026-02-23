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

### Effects And Boundaries In Atlasctl

- Only `atlasctl.core.fs` is allowed to own direct file-write primitives.
- Only `atlasctl.core.exec` is allowed to own subprocess execution primitives.
- Only `atlasctl.core.env` is allowed to own direct environment-variable reads/writes.
- Only `atlasctl.core.process` is allowed to own process lifecycle/retry/timeouts.
- Only `atlasctl.core.network` is allowed to own direct network request primitives.
- Any temporary exception must be listed in `configs/policy/effect-boundary-exceptions.json` with a reason.

See `docs/atlasctl/BOUNDARIES.md` for effect boundaries, invariants, output contracts, and policy enforcement rules.

### Ops Boundary (Control Plane)

- `packages/atlasctl/src/atlasctl/commands/ops/**` owns ops execution orchestration.
- `ops/` stores manifests, contracts, schemas, fixtures, and test inputs; it is not a public wrapper surface.
- `commands/ops/**` may depend on `core/`, `contracts/`, `reporting/`, `registry/`, `commands/_shared.py`, and intra-ops modules.
- `commands/ops/**` must not import `cli/` modules directly.
- Internal migration glue belongs under `commands/ops/internal/**` and is excluded from public ops help/docs.

See:
- `packages/atlasctl/docs/control-plane/ops-execution-model.md`
- `packages/atlasctl/docs/control-plane/ops-taxonomy.md`

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
