# Architecture Budgets

This document defines directory density budgets for `atlasctl` and the rationale.

## Budget Policy

Canonical SSOT is `packages/atlasctl/pyproject.toml` under `[tool.atlasctl.budgets]`.

Supported keys:
- `max_py_files_per_dir`
- `max_modules_per_dir`
- `max_shell_files_per_dir`
- `max_total_loc_per_dir`
- `max_total_bytes_per_dir`
- `max_imports_per_file`
- `max_public_symbols_per_module`
- `max_branch_keywords_per_file` (complexity heuristic)

Warnings are emitted when usage is within 10% of budget. Failing above budget is a hard error.

## Directory Classes

- `src/atlasctl/core/*`: stricter budgets because core should remain small and stable.
- `src/atlasctl/checks/layout/*`: higher allowance because this area hosts many policy checks, but still bounded.
- `ops/vendor/layout-checks/*`: explicit shell-count cap for quarantined transitional shell probes.
- `src/atlasctl/checks/layout/policies/*`: stricter module-count cap to force continued split into first-class layout domains.
- `src/atlasctl/legacy/*`: enforcement disabled; no new code should be added and migration should reduce density over time.

## Exceptions

- `packages/atlasctl/src/atlasctl/legacy/docs_runtime_chunks`
: Legacy compatibility runtime shards with dynamic loading.

## Commands

Use these culprits reports:
- `atlasctl policies culprits modules-per-dir`
- `atlasctl policies culprits py-files-per-dir`
- `atlasctl policies culprits shell-files-per-dir`
- `atlasctl policies culprits dir-loc`
- `atlasctl policies culprits imports-per-file`
- `atlasctl policies culprits public-symbols-per-file`
- `atlasctl policies culprits complexity-heuristic`
- `atlasctl policies culprits-biggest-files --limit 20`
- `atlasctl policies culprits-biggest-dirs --limit 20`

Each output includes directory, measured count, budget, status, and top offenders.

Complexity heuristic is gated only for `packages/atlasctl/src/atlasctl/core/` and `packages/atlasctl/src/atlasctl/cli/`.
