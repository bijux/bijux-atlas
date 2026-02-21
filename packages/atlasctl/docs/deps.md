# Dependency Workflow

Workflow: pip-tools (requirements.in + requirements.lock.txt)

`atlasctl` keeps package metadata in `pyproject.toml` and uses `pip-tools` inputs/locks for deterministic local and CI installs.

## Files

- `packages/atlasctl/requirements.in`: human-edited dependency input.
- `packages/atlasctl/requirements.lock.txt`: resolved lock file used for deterministic installs.
- `packages/atlasctl/pyproject.toml`: package metadata and tool configuration.

## Policy

- Update lockfiles when dependency declarations change.
- Do not introduce alternate lock workflows in this package without updating this document and checks.
