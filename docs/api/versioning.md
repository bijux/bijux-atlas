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

## Related Pages

- [Compatibility Policy](compatibility.md)
- [V1 Surface](v1-surface.md)
