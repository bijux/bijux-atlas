# API versioning

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define path versioning and deprecation rules clearly.

## Versioning rules

- API major version is path-based: `/v1/...`.
- Dataset release identifiers are data-version controls, not API-major controls.
- Existing `v1` behavior evolves additively only.

## Deprecation rules

- Deprecations must include a replacement path.
- Removal occurs only in a future major API version.

## Next steps

- [Compatibility](compatibility.md)
- [Deprecation lifecycle](deprecation.md)
- [Reference contracts](../reference/contracts/index.md)
