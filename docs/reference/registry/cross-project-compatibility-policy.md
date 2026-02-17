# Cross-Project Compatibility Policy

Concept IDs: concept.compatibility-matrix

- Owner: `bijux-atlas-api`

Canonical page: [`docs/contracts/compatibility.md`](../../contracts/compatibility.md)

## What

Pointer page for cross-project compatibility policy semantics.

## Why

Avoids duplicate policy text across registry and contracts sections.

## Scope

References compatibility guarantees across umbrella and plugins.

## Non-goals

Does not restate the canonical compatibility contract.

## Contracts

All normative statements are defined in the canonical page.

## Failure modes

Duplicated policy text can drift and cause contradictory guarantees.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Compatibility Contract](../../contracts/compatibility.md)
- [Registry Index](INDEX.md)
- [Terms Glossary](../../_style/terms-glossary.md)
