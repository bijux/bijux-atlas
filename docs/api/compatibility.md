# API Compatibility

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define API compatibility guarantees without duplicating product policy narrative.

## Compatibility guarantees

- Existing documented API behavior remains stable for `v1`.
- Additive changes are allowed when they do not break existing clients.
- Breaking changes require a new major API version.

## Canonical promise

Product-level compatibility narrative lives in [Product Compatibility Promise](../product/compatibility-promise.md).

## Next

- [Versioning](versioning.md)
- [Product Compatibility Promise](../product/compatibility-promise.md)
