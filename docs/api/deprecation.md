# API Deprecation

- Owner: `api`
- Stability: `stable`

## Policy

- Deprecations in v1 are signaled as:
- OpenAPI `deprecated: true`
- Documentation note in `docs/api/V1_SURFACE.md`
- `Deprecation` + `Sunset` response headers when applicable

## Timeline

- Deprecation announcement: immediate in docs + contract.
- Grace period: minimum two minor releases.
- Removal: only in next major API version (not within v1).
