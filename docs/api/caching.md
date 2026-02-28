# Caching Semantics

Owner: `api-contracts`  
Type: `policy`  
Surface version: `v1`  
Reason to exist: define one canonical caching contract for API responses.

## Contract

- ETag is derived from dataset artifact identity plus normalized request identity.
- `If-None-Match` with matching ETag returns `304`.
- Immutable dataset reads use cache headers suitable for long-lived caching.

## Example

```bash
curl -i -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
