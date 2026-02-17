# Plugin Versioning And Release Independence

Concept IDs: concept.compatibility-matrix

- Owner: `bijux-atlas-cli`

Canonical page: [`docs/contracts/compatibility.md`](../../contracts/compatibility.md)

## What

Pointer page for plugin versioning and release cadence compatibility policy.

## Why

Maintains one canonical definition of compatibility semantics.

## Scope

Covers umbrella and plugin version-range expectations.

## Non-goals

Does not duplicate canonical compatibility contract prose.

## Contracts

Normative versioning guarantees are defined in the canonical page.

## Failure modes

Conflicting policy wording across pages causes operator confusion.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Compatibility Contract](../../contracts/compatibility.md)
- [Plugin Contract](../../contracts/plugin/spec.md)
- [Registry Index](INDEX.md)
