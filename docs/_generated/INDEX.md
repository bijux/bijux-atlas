# Generated Docs Index

- Owner: `docs-governance`

## What

Defines policy for generated documentation under `docs/_generated/`.

## Why

Generated outputs must remain deterministic and non-manual.

## Scope

Applies to all generated markdown contract mirrors.

## Non-goals

Does not define handwritten narrative docs.

## Contracts

- Generated docs live only under `docs/_generated/`.
- Manual edits to generated docs are forbidden.
- Regeneration is required when SSOT contracts change.

## Failure modes

Manual edits or stale generated files create contract drift.

## How to verify

```bash
$ make docs-freeze
```

Expected output: no generated docs drift.

## See also

- [Generated Contracts Index](contracts/INDEX.md)
- [Generated Make Targets](make-targets.md)
- [Generated Ops Surface](ops-surface.md)
- [Generated Config Keys](config-keys.md)
- [Generated Contracts Catalog](contracts-index.md)
- [Generated Runbook Map Index](runbook-map-index.md)
- [Contracts Index](../contracts/contracts-index.md)
- [Terms Glossary](../_style/terms-glossary.md)
- [OpenAPI](openapi/INDEX.md)
