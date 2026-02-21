# Budget Policies and Refactoring

`atlasctl` enforces structural budgets to keep modules maintainable and reviewable.

## Why budgets exist

- Prevent hotspot directories from becoming dumping grounds.
- Force early refactors instead of late rewrites.
- Keep CI failures actionable with deterministic culprit reports.

## Active budget gates

- `max_py_files_per_dir`: cap Python files in one directory.
- `max_modules_per_dir`: cap non-`__init__.py` modules in one directory.
- `max_dir_entries_per_dir`: cap files+dirs in one directory.
- `max_modules_per_domain`: cap modules in each top-level `src/atlasctl/<domain>`.
- `max_loc_per_file`: cap file size.
- `max_loc_per_dir`: cap total directory LOC.
- `max_tree_depth`: cap nesting depth under `packages/atlasctl/src/atlasctl`.

## Directory Budget SSOT

Scope:

- `packages/atlasctl/src/atlasctl/**`
- `packages/atlasctl/tests/**`

Hard limits:

- Max `10` `.py` files per directory, excluding `__init__.py`.
- Max `10` total directory entries (files + dirs), excluding `__pycache__`.
- Entry budgets do not exclude `README.md`.

Tests use the same default thresholds to keep test trees clean and split by intent.

## Exceptions policy

- Exceptions are allowed only via `configs/policy/BUDGET_EXCEPTIONS.yml`.
- Every exception must include `path`, `owner`, `reason`, and `expires_on`.
- Exception count is capped (`max_exceptions` in `BUDGET_EXCEPTIONS.yml`).
- Every exception path must be documented in `packages/atlasctl/docs/architecture.md`.
- Expired exceptions fail CI.

## Culprit reports

- Worst directories by LOC: `atlasctl policies culprits-biggest-dirs --report json`
- Worst files by LOC: `atlasctl policies culprits-biggest-files --report json`
- Budget suite gate: `atlasctl policies culprits-suite --report json`

## Gradual tightening

- Current thresholds are versioned in `configs/policy/atlasctl-budgets-baseline.json`.
- Budgets may not be loosened without `configs/policy/budget-loosening-approval.json`.
- Tightening is expected over time by lowering defaults and removing exceptions.

## Refactoring workflow

1. Run `atlasctl policies culprits-suite --report json` and identify failing paths.
2. Split by intent boundaries, not by arbitrary file count.
3. Move shared logic into small helpers and keep command wiring thin.
4. Re-run the suite and keep deterministic ordering in reports.

Required reading when a budget fails:

- `packages/atlasctl/docs/architecture/how-to-split-a-module.md`
