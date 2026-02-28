# Compatibility Policy

Owner: `api-contracts`  
Type: `policy`  
Surface version: `v1`  
Reason to exist: provide one canonical compatibility contract for API consumers.

## v1 Compatibility Guarantees

- Existing documented endpoints and fields remain stable.
- Additive changes are allowed when backward compatible.
- Breaking behavior changes require a new major API version.

## Compatibility Limits

- Undocumented behavior is not guaranteed.
- Experimental surfaces are excluded from long-term compatibility promises.

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/openapi.json' >/dev/null
```

## Related References

- [Product Compatibility Promise](../product/compatibility-promise.md)
- [Errors Reference](../reference/errors.md)
- [Schemas Reference](../reference/schemas.md)
