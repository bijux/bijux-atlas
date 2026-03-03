# Documentation Generation Model

- Owner: `bijux-atlas-platform`
- Audience: `contributors`

## Decision

Generated documentation artifacts are committed under `docs/_internal/generated/`.
This repository does not use a committed `docs/_generated/` surface.

## Required Outputs

- `docs/_internal/generated/search-index.json`
- `docs/_internal/generated/sitemap.json`
- `docs/_internal/generated/docs-inventory.md`
- `docs/_internal/generated/topic-index.json`

## Regeneration

- `cargo run -q -p bijux-dev-atlas -- docs audit --allow-write`
- `cargo run -q -p bijux-dev-atlas -- docs index --allow-write`

## Build Rule

Documentation build and governance checks must fail when required generated outputs are missing.
