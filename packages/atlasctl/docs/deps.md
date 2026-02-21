# Dependency Workflow

Workflow: pip-tools (requirements.in + requirements.lock.txt)

`atlasctl` keeps package metadata in `pyproject.toml` and uses `pip-tools` inputs/locks for deterministic local and CI installs.

## Files

- `packages/atlasctl/requirements.in`: canonical dependency input.
- `packages/atlasctl/requirements.lock.txt`: deterministic lock derived from `requirements.in`.
- `packages/atlasctl/pyproject.toml`: package metadata and tool configuration.

## Policy

- Route B is authoritative for this package.
- Do not add `uv.lock` or other requirements files in package root.
- Update lockfile whenever dependency declarations change.
- Keep tool and test configuration in `pyproject.toml`; keep `requirements.in` only as the explicit lock input for deterministic installs.

## Commands

- `python -m atlasctl.cli deps lock`
- `python -m atlasctl.cli deps export-requirements`
- `python -m atlasctl.cli deps sync`
- `python -m atlasctl.cli deps check-venv`
- `python -m atlasctl.cli deps cold-start --runs 3 --max-ms 500`

## Dependency Ownership

| Dependency | Owner | Justification |
| --- | --- | --- |
| `pytest==8.3.5` | `platform` | Unit test runner and fixtures. |
| `pytest-timeout==2.3.1` | `platform` | Prevent hanging test runs in CI/local gates. |
| `ruff==0.13.2` | `platform` | Lint and format enforcement. |
| `jsonschema==4.25.1` | `platform` | Contract/schema validation utilities. |
| `hypothesis==6.138.14` | `platform` | Property-based tests for path/write policies. |
| `mypy==1.17.1` | `platform` | Static type checks for core/contracts. |
