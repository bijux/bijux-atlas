# Compatibility Policy

Owner: `api-contracts`  
Type: `policy`  
Reason to exist: provide one canonical compatibility contract for API consumers.

## v1 Compatibility Guarantees

- Existing documented endpoints and fields remain stable.
- Additive changes are allowed when backward compatible.
- Breaking behavior changes require a new major API version.

## Compatibility Limits

- Undocumented behavior is not guaranteed.
- Experimental surfaces are excluded from long-term compatibility promises.

## Related Pages

- [Versioning Policy](versioning.md)
- [Errors](errors.md)
- [Schemas Reference](../reference/schemas.md)
