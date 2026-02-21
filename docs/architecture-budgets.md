# Architecture Budgets

This document defines directory density budgets for `atlasctl` and the rationale.

## Budget Policy

Canonical SSOT is `packages/atlasctl/pyproject.toml` under `[tool.atlasctl.budgets]`.

Supported keys:
- `max_py_files_per_dir`
- `max_modules_per_dir`
- `max_total_loc_per_dir`
- `max_total_bytes_per_dir`

Warnings are emitted when usage is within 10% of budget. Failing above budget is a hard error.

## Directory Classes

- `src/atlasctl/core/*`: stricter budgets because core should remain small and stable.
- `src/atlasctl/checks/layout/*`: higher allowance because this area hosts many policy checks, but still bounded.
- `src/atlasctl/legacy/*`: enforcement disabled; no new code should be added and migration should reduce density over time.

## Exceptions

- `packages/atlasctl/src/atlasctl/legacy/docs_runtime_chunks`
: Legacy compatibility runtime shards with dynamic loading.

## Commands

Use these culprits reports:
- `atlasctl policies culprits modules-per-dir`
- `atlasctl policies culprits py-files-per-dir`
- `atlasctl policies culprits dir-loc`

Each output includes directory, measured count, budget, status, and top offenders.
