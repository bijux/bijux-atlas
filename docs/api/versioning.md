# Versioning Policy

Owner: `api-contracts`  
Type: `policy`  
Surface version: `v1`  
Reason to exist: define stable API versioning and change constraints.

## Rules

- API versioning is path-based (`/v1/...`).
- Dataset release is data identity, not API versioning.
- v1 changes are additive-only for existing documented behavior.

## Deprecation

- Deprecated surfaces must be explicitly marked.
- Grace windows are announced before removal in future major versions.

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/version'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
