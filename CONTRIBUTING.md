# Contributing

## Scope
Bijux Atlas is shipped as a Rust workspace with two primary surfaces:
- `bijux-atlas`: runtime product crates (core, model, query, store, ingest, api, server, cli)
- `bijux-dev-atlas`: control-plane crates for checks, docs/config governance, and ops orchestration

## Development Contract
- Use only `make` targets or `bijux dev atlas ...` commands.
- Keep outputs deterministic and write artifacts only under `artifacts/`.
- Update docs/contracts when command, schema, policy, API, or output surfaces change.
- Keep commit messages in Conventional/Commitizen format with clear intent.

## Local Validation
Run before opening a pull request:
- `make fmt`
- `make lint`
- `make test`
- `make audit`
- `make check`

## Pull Request Requirements
- Include focused, logical commits.
- Include validation evidence in PR description.
- Keep documentation and registry in lockstep.

## Ownership
See `.github/CODEOWNERS` and `docs/governance/DOCS_OWNERSHIP.md`.
