# Atlasctl Checks

`atlasctl.checks` is the canonical checks subsystem.

## Single Source of Truth

- Runtime check definitions are sourced from Python registry modules in `atlasctl/checks/registry.py`.
- Runtime selector/list/explain paths consume `atlasctl.checks.registry.ALL_CHECKS`.
- `REGISTRY.generated.json` are generated artifacts, not runtime inputs.
- Check execution is centralized in `atlasctl.checks.runner`.

## Effect Policy

- Checks are default-deny for side effects.
- Default allowed effect is `fs_read`.
- Additional effects (`fs_write`, `subprocess`, `git`, `network`) must be declared by each check and explicitly enabled by command capabilities.
- Evidence writes must stay under `artifacts/evidence/<run-id>/...`.

## Add a Check

1. Implement check logic in `checks/tools/` or a flat `checks/domains/*.py` module.
2. Return structured violations (or legacy tuple where still in migration).
3. Register it in the domain `CHECKS` export with canonical `checks_<domain>_<area>_<intent>` id, owner, tags, effects, and budget.
4. Regenerate registry artifacts.
5. Add or update tests and goldens.

### Minimal CheckDef Example

```python
CheckDef(
    "checks_repo_root_shape",
    "repo",
    "enforce repository root shape contract",
    500,
    run_root_shape,
    owners=("platform",),
    tags=("repo", "required"),
    effects=("fs_read",),
)
```

## Final Tree

- `checks/__init__.py`
- `checks/model.py`
- `checks/registry.py`
- `checks/selectors.py`
- `checks/policy.py`
- `checks/runner.py`
- `checks/report.py`
- `checks/gen_registry.py`
- `checks/domains/*.py`
- `checks/tools/*.py`

Legacy trees (`checks/layout`, `checks/repo`, `checks/registry` package) are migration-only and blocked by internal policy gates.

## Selectors

- `atlasctl check run` supports filtering by:
  - `--domain`
  - `--category`
  - `--id` / `--select` / `-k`
  - `--tag` / `--exclude-tag`
  - `--owner`
  - `--slow` / `--fast`
  - `--include-internal`
  - `--changed-only`

Selectors are resolved before execution and flow through the same runner/report path for `check` and `lint`.
