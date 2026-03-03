# API versioning

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
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
