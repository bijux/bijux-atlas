# Dependency Workflow

Workflow: pip-tools (requirements.in + requirements.lock.txt)

`pyproject.toml` is the dependency/configuration center.
`requirements.in` and `requirements.lock.txt` are derived lock workflow artifacts for deterministic local and CI installs.

## Files

- `packages/atlasctl/requirements.in`: canonical dependency input.
- `packages/atlasctl/requirements.lock.txt`: deterministic lock derived from `requirements.in`.
- `packages/atlasctl/pyproject.toml`: package metadata and tool configuration.

## Policy

- Chosen workflow: Route B (`requirements.in` compiled to `requirements.lock.txt`).
- Do not add `uv.lock` or other requirements files in package root.
- Update lockfile whenever dependency declarations change.
- Keep tool and test configuration in `pyproject.toml`; keep `requirements.in` only as the explicit lock input for deterministic installs.

## Compile Command

- Canonical lock refresh command:
  - `python -m atlasctl.cli deps lock`

## Hash Policy

- Lock entries are exact version pins.
- Per-package hash pins are currently not required for this internal package workflow.
- If hashes are adopted later, enforce them through the same `deps lock` command and CI lane.

## Update Cadence

- Refresh lock on any dependency declaration change in `pyproject.toml`.
- Minimum cadence: weekly during active development.
- Required refresh before release tagging and when CI dependency checks fail.

## Exceptions

- Dependency-policy exceptions must be recorded in:
  - `configs/policy/dependency-exceptions.json`
- Every exception entry must include a justification string.

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
| `tomli>=2.0.0; python_version < "3.11"` | `platform` | Compatibility fallback for TOML parsing on Python <3.11. |
