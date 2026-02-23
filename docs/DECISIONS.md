# Atlasctl Decisions (ADR-lite)

Date: 2026-02-21
Scope: `packages/atlasctl/` architecture and boundary contract.

## Initial Decisions

1. Canonical package shape is documented as **Target Layout v1** in `docs/ARCHITECTURE.md`.
2. Allowed top-level items under `packages/atlasctl/` are exactly:
   `src/`, `tests/`, `docs/`, `pyproject.toml`, `README.md`, `LICENSE`.
3. Lockfile workflow choice: **pyproject-first + pip-compatible lock discipline** (current repo-compatible path).
4. Lock file policy: **required** for CI determinism.
5. Canonical CLI entry: `python -m atlasctl` plus installed `atlasctl` console script.
6. Canonical check system: **registry-based** under `atlasctl/checks/*`.
7. Canonical command system: **`atlasctl/commands/*` with `configure()/run()`**.
8. Legacy policy (pre-1.0 hard reset): legacy code must be deleted, not preserved.
9. Observability namespace decision: **`observability/` is canonical**; `obs/` remains compatibility-only.
10. Filesystem boundary decision: **`core/fs.py` is canonical**; top-level `fs.py` is non-canonical legacy surface.
11. Process execution boundary decision: **`core/exec.py` is canonical**; top-level `subprocess.py` is non-canonical legacy surface.
12. ADR-lite mechanism: this file (`docs/DECISIONS.md`) is the living decision register for this package.

## Enforcement Intent

- New modules must not be introduced under deprecated duplicate namespaces.
- New commands must register via `cli` + `commands` canonical path.
- New checks must register in the `checks` registry and be discoverable via `atlasctl check list`.
- CI and local developer targets should consume lock-resolved dependencies.
- Contracts boundary:
  - `atlasctl/contracts/` is the public schema catalog, schema IDs, and output contract layer.
  - `atlasctl/core/schema/` contains internal schema/yaml helper utilities only (no contract ownership).
  - `commands/` and `checks/` must import schemas from `atlasctl/contracts`, not `atlasctl/core/schema`.
- Integration boundary:
  - `atlasctl/adapters/` is the canonical external integration layer.
  - `atlasctl/core/integration/` is deprecated and must not be reintroduced.
- Context boundary:
  - `RunContext` construction is centralized in `atlasctl/core/context.py` via `RunContext.from_args`.
