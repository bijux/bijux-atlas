# Ops SSOT

- Owner: `bijux-atlas-platform`
- Audience: `contributors`
- Stability: `stable`

## Scope

`ops/` is the operational source of truth for executable operations content.

## Allowed Root Markdown

- `ops/README.md`
- `ops/CONTRACT.md`
- `ops/INDEX.md`
- `ops/ERRORS.md`
- `ops/SSOT.md`

## Forbidden Markdown Shape

- Nested markdown under `ops/**`
- Runbooks, onboarding guides, templates, and narrative walkthroughs inside domain subtrees
- Markdown mirrors for data that already has a JSON, YAML, TOML, or schema authority

## Linking Boundaries

- `docs/` may reference stable operational specs in the five root docs or machine-readable ops authorities.
- `ops/` may reference stable public docs pages when necessary.
- `ops/` must not use `docs/` internals as canonical user guidance.

## Rationale

Nested markdown in `ops/` made the tree hard to trust by path. Root docs now explain the system once, and the rest of `ops/` stays machine-readable so ownership and SSOT are obvious from the tree.
