# Dependency Update Policy

Dependency updates are constrained to deterministic, reviewable paths.

## Rules
- Dependabot proposes weekly `cargo` and `github-actions` updates.
- Lockfile refresh is performed by `.github/workflows/dependency-lock.yml`.
- Manual dependency bumps must include lockfile updates and pass full CI.
- Release branches must use `--locked` builds.

## Why
This reduces surprise transitive drift and keeps updates auditable.
