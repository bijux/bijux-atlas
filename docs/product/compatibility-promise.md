# Compatibility Promise

- Owner: `bijux-atlas-product`

## What

Upgrade guarantees for API, artifact, and cursor contracts.

## Why

Consumers need predictable upgrades without query rewrites.

## Scope

Applies to v1 surfaces.

## Non-goals

Does not promise behavior for undocumented fields or experimental registries.

## Contracts

- API: existing paths and documented fields remain stable within v1.
- Artifacts: manifest and dataset layout remain backward-readable within v1.
- Cursor: decoder remains backward-compatible for previous v1 cursor versions.
- Error codes: existing codes remain valid identifiers.

## Failure modes

Breaking changes without version bump are contract violations and must fail CI.

## How to verify

```bash
$ ./scripts/contracts/check_breaking_contract_change.py
$ make openapi-drift
```

Expected output: no breaking change detected and OpenAPI drift check passes.

## See also

- [Contracts Compatibility](../contracts/compatibility.md)
- [API Versioning](../reference/evolution/api-versioning.md)
- [JSON Compatibility](../reference/compatibility/json-wire-compatibility.md)
