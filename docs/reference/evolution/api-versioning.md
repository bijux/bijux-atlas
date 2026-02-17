# API Versioning

Atlas API versioning is path-based (`/v1/...`).

Rules:
- v1 is additive-only for fields and endpoints.
- Existing fields are never removed or renamed in v1.
- Removed behavior requires:
  - prior deprecation annotation in OpenAPI
  - migration note in docs/evolution/
  - major version bump (`/v2`).
- Cursor decoding is backward-compatible within v1.
- JSON response field order is stable and deterministic for the same request.

Deprecation:
- Deprecated endpoints/params use OpenAPI `deprecated: true`.
- Deprecations remain available for at least one minor release cycle.
