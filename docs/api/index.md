# API

Owner: `api-contracts`  
Type: `guide`  
Surface version: `v1`  
Reason to exist: define how to call the stable Atlas API surface.

## How To Call The API

1. Select dataset identity explicitly: `release/species/assembly`.
2. Call versioned endpoints under `/v1/...`.
3. Use cursor pagination for list responses.
4. Handle stable error envelopes and codes.

## Canonical API Pages

- [V1 Surface](v1-surface.md)
- [Quick Reference](quick-reference.md)
- [Client Behavior Expectations](client-behavior.md)
- [Versioning Policy](versioning.md)
- [Pagination Guide](pagination.md)
- [Compatibility Policy](compatibility.md)
- [Error Handling](errors.md)
- [Caching Semantics](caching.md)
- [Deprecation Policy](deprecation.md)
- [Default Field Set](default-field-set.md)
- [Troubleshooting](troubleshooting.md)

## Reference Surfaces

- [Error Codes Reference](../reference/errors.md)
- [Schemas Reference](../reference/schemas.md)
- [Commands Reference](../reference/commands.md)

## What To Read Next

- [Start Here](../start-here.md)
- [Operations](../operations/index.md)
