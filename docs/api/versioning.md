# API Versioning

- Owner: `api`
- Stability: `stable`

- API version is path-based and frozen to `/v1/...` for this line.
- Dataset release (`release=...`) is dataset identity, not API versioning.
- v1 compatibility rules:
  - responses are additive-only
  - new params must be optional
  - new endpoints are additive
- Deprecation policy uses OpenAPI `deprecated` plus `Deprecation`/`Sunset` headers when applicable.
- Compatibility mapping:
  - deprecated: `/v1/releases/{release}/species/{species}/assemblies/{assembly}`
  - canonical: `/v1/datasets/{release}/{species}/{assembly}`
- v2 is a non-goal right now; when introduced it will use `/v2/...` with a separate compatibility contract.
