# Ops SSOT

- Owner: `bijux-atlas-platform`
- Audience: `contributors`
- Stability: `stable`

## Scope

`ops/` is the operational source of truth for executable operations content.

## Allowed Operational Content

- specifications
- runbooks
- policies
- evidence references
- canonical stubs

## Forbidden Content

- narrative documentation
- onboarding guides

Narrative and onboarding content must live in `docs/`.

## Required Front Matter

Every `ops/*.md` document must declare `Doc-Class` front matter.
Allowed values are `spec`, `runbook`, `policy`, `evidence`, and `stub`.

## Linking Boundaries

- `docs/` may reference stable operational specs in `ops/`.
- `ops/` may reference stable public docs pages when necessary.
- `ops/` must not use `docs/_internal/` as canonical user guidance.
