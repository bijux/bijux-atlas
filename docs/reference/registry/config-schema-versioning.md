# Bijux Config Schema Versioning

- Owner: `bijux-atlas-store` + `bijux-atlas-server`

## What

Catalog/config schemas are explicit, versioned contracts.

- `schema_version` is required in every versioned config payload.
- Versioning policy:
  - patch: docs-only clarification, no wire format change.
  - minor: additive fields only, backwards compatible.
  - major: any removal/rename/semantic break.
- Unknown fields are rejected by default (`deny_unknown_fields` policy).

## Why

Prevents silent drift across CLI, store, and server runtimes.

## Scope

Applies to catalog/config schemas consumed by store and server.

## Non-goals

Does not define release cadence or product API versioning.

## Contracts

- Any schema bump requires:
  - updated docs and compatibility note
  - tests covering decode behavior at previous/current version
  - changelog entry in release notes

## Failure modes

- Missing `schema_version`: reject.
- Unsupported major version: reject.
- Unknown field in strict schema: reject.

## How to verify

```bash
$ make contracts
```

Expected output: schema/version drift checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
