# Atlasctl Checks

`atlasctl.checks` is the canonical checks subsystem.

## Single Source of Truth

- Runtime check definitions are sourced from Python registry modules under `atlasctl/checks/registry/`.
- `REGISTRY.toml` and `REGISTRY.generated.json` are generated artifacts, not the runtime source of truth.
- Check execution is centralized in `atlasctl.checks.runner`.

## Effect Policy

- Checks are default-deny for side effects.
- Default allowed effect is `fs_read`.
- Additional effects (`fs_write`, `subprocess`, `git`, `network`) must be declared by each check and explicitly enabled by command capabilities.
- Evidence writes must stay under `artifacts/evidence/<run-id>/...`.

## Add a Check

1. Implement check logic in a domain module.
2. Return structured violations (or legacy tuple where still in migration).
3. Register it in the domain `CHECKS` export with:
   - canonical `checks_<domain>_<area>_<intent>` id
   - `owner`
   - `category`
   - `result_code`
   - `effects`
4. Regenerate registry artifacts.
5. Add or update tests and goldens.

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
