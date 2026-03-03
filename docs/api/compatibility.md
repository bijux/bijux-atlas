# API compatibility

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define API compatibility guarantees without duplicating product policy narrative.

## Compatibility guarantees

- Documented `v1` paths, field names, cursor semantics, and stable error codes remain compatible within `v1`.
- Additive fields and additive endpoints are allowed when existing clients continue to work unchanged.
- Breaking wire changes require a new API major version.

## Canonical promise

The product-level promise lives once in [Product compatibility promise](../product/compatibility-promise.md). This page only explains what that promise means for API consumers.

## Reference sources

- Contract mapping: [Reference contracts compatibility](../reference/contracts/compatibility.md)
- Versioning rules: [API versioning](versioning.md)
- Verification: [Compatibility test plan](compatibility-test-plan.md)

## Next steps

- [Versioning](versioning.md)
- [Product compatibility promise](../product/compatibility-promise.md)
