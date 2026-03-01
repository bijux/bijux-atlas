# Deprecation lifecycle

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define how stable API surfaces are deprecated and how consumers should react.

## Lifecycle

1. A stable endpoint is marked deprecated in published API docs and contract metadata.
2. The replacement path or behavior is documented before the deprecated surface is removed.
3. The deprecated surface remains available for the rest of the current major version unless an explicit security exception is announced.
4. Removal happens only in a future major API version.

## Current example

`GET /v1/genes/count` and `GET /v1/releases/{release}/species/{species}/assemblies/{assembly}` are documented as deprecated and point to their canonical replacements in the `v1` surface.

## Verification

Check the current deprecation markers in [V1 surface](v1-surface.md) and [Reference contracts endpoints](../reference/contracts/endpoints.md).

## Next steps

- [Versioning](versioning.md)
- [Compatibility](compatibility.md)
