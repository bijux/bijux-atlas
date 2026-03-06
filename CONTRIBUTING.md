# Contributing

## Scope
Bijux Atlas ships two command surfaces:
- `bijux atlas` for runtime product actions
- `bijux dev atlas` for repo checks, contracts, docs, configs, and ops governance

## Repository Rules
Canonical repo law source lives in `docs/_internal/contracts/repo-laws.md`.
Do not restate law text in root docs.
For structural boundaries and crate/root-directory rationale, use `docs/architecture/why-this-structure-exists.md` as the canonical reference and update it when shape changes.

## Local Validation
- `bijux dev atlas check doctor --format text`
- `make fmt`
- `make lint`
- `make test`
- `make check`

## Ownership
See `.github/CODEOWNERS` and `docs/reference/repo-map.md`.
