# Deprecation Policy

Owner: `api-contracts`  
Type: `policy`  
Surface version: `v1`  
Reason to exist: define endpoint deprecation rules for stable API consumers.

## Policy

- Deprecation is announced in docs and contract metadata.
- Deprecated endpoints are marked before any removal in future major versions.
- Existing v1 compatibility guarantees remain active during deprecation windows.

## Example

```bash
curl -i -fsS 'http://127.0.0.1:8080/v1/datasets'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
