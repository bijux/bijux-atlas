# Budget Policies and Refactoring

`atlasctl` enforces structural budgets to keep modules maintainable and reviewable.

## Why budgets exist

- Prevent hotspot directories from becoming dumping grounds.
- Force early refactors instead of late rewrites.
- Keep CI failures actionable with deterministic culprit reports.

## Active budget gates

- `max_py_files_per_dir`: cap Python files in one directory.
- `max_modules_per_dir`: cap non-`__init__.py` modules in one directory.
- `max_modules_per_domain`: cap modules in each top-level `src/atlasctl/<domain>`.
- `max_loc_per_file`: cap file size.
- `max_loc_per_dir`: cap total directory LOC.
- `max_tree_depth`: cap nesting depth under `packages/atlasctl/src/atlasctl`.

## Exceptions policy

- Exceptions are allowed only via `[[tool.atlasctl.budgets.exceptions]]` in `packages/atlasctl/pyproject.toml`.
- Every exception must include both `path` and `reason`.
- Every exception path must be documented in `packages/atlasctl/docs/architecture.md`.

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
