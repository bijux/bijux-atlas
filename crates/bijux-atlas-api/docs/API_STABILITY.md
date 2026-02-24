# API Stability and Versioning

## Contract Scope

The API contract includes:
- OpenAPI output from `openapi_v1_spec`.
- Public DTOs in `src/dto.rs`.
- Error schema and code set (`ApiError`, `ApiErrorCode`).
- Compatibility behavior in `src/compat.rs`.

The contract excludes internal helper functions and test-only fixtures.

## Versioning Rules

- `v1` routes are additive-only for request/response shape.
- Existing field semantics are stable once shipped.
- Error `code` values are machine-stable.
- Breaking changes require a new path namespace (for example `v2`).

## Compatibility Layer

Legacy `v0.x` route family is represented as explicit redirects:
- Source: `/v1/releases/{release}/species/{species}/assemblies/{assembly}`
- Canonical: `/v1/datasets/{release}/{species}/{assembly}`
- Status: `308`

## OpenAPI Pinning

`OPENAPI_V1_PINNED_SHA256` pins canonical OpenAPI bytes. Any intentional contract change must update:
- `configs/openapi/v1/openapi.snapshot.json`
- `OPENAPI_V1_PINNED_SHA256`
- relevant compatibility tests
