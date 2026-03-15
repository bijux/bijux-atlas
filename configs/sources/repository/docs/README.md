# Docs tooling configs

- Owner: `docs-governance`
- Purpose: configure documentation linters, build tooling, and reader-surface quality rules.
- Consumers: `bijux-dev-atlas` docs commands, MkDocs-adjacent tooling, and docs CI lanes.
- Update workflow: change the specific tooling input, then rerun docs contracts and strict docs builds.

Deterministic install policy:
- Node tooling is pinned by `package-lock.json`.
- Install via control-plane docs workflows (`bijux dev atlas docs ...`) so lockfile resolution remains reproducible.
