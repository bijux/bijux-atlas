# Concept Registry

- Owner: `docs-governance`

## What

Defines canonical concepts and their single source pages.

## Why

Prevents duplicated concept narratives and conflicting policy statements.

## Scope

Applies to concept-bearing docs listed in `docs/_internal/governance/metadata/concepts.yml`.

## Non-goals

Does not replace endpoint, metric, or error SSOT registries.

## Contracts

- Each concept ID must exist in `docs/_internal/governance/metadata/concepts.yml`.
- Each concept has exactly one canonical page.
- Non-canonical pages must use pointer format and link to canonical page.
- Docs introducing new concepts must update the registry first.

## Failure modes

Duplicate canonical ownership or missing concept declarations create contract drift.

## How to verify

```bash
$ bijux dev atlas docs check --report text
$ bijux dev atlas docs generate --report text
$ make docs
```

Expected output: concept registry checks and docs build pass.

## See also

- [Concept IDs](concept-ids.md)
- [Structure Templates](structure-templates.md)
- Generated concept registry is available from [Docs Dashboard](../docs-dashboard.md).
