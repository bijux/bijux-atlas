# API Surface Index

- Owner: `api`
- Stability: `stable`

## What

Single entrypoint for v1 API surface, dataset selection rules, pagination, filters, and error model.

## Why

Keeps API contracts discoverable in one canonical location and prevents drift across per-endpoint docs.

## Scope

Applies to all `docs/api/*.md` pages and generated OpenAPI contract views.

## Non-goals

Does not duplicate endpoint-level details already defined by `docs/contracts/ENDPOINTS.json`.

## Contracts

- API contract source: `docs/contracts/ENDPOINTS.json`
- Generated OpenAPI: `docs/_generated/openapi/openapi.generated.json`
- Contract checks: `make api-contract-check`, `make openapi-drift`

## Canonical Entry Points

- Surface list: `docs/api/V1_SURFACE.md`
- Versioning: `docs/api/versioning.md`
- Pagination: `docs/api/pagination.md`
- Caching/ETag: `docs/api/caching.md`
- Errors: `docs/api/errors.md`
- Compatibility/deprecation: `docs/api/COMPATIBILITY.md`, `docs/api/DEPRECATION.md`

## Dataset Selection Rules

- Dataset selection is explicit on all gene/transcript/sequence query endpoints.
- Canonical dataset resource: `/v1/datasets/{release}/{species}/{assembly}`.
- Legacy `/v1/releases/{release}/species/{species}/assemblies/{assembly}` is deprecated and redirects.

## Filter and Query Contracts

- Filter grammar SSOT: `docs/contracts/FILTERS.json`.
- Endpoint and parameter SSOT: `docs/contracts/ENDPOINTS.json`.
- Query preflight classifier: `POST /v1/query/validate`.

## Response Envelope Rules

- List endpoints: `{ items, stats }` in `data` plus cursor in `page/links`.
- Single-resource endpoints: `{ item }` in `data`.

## Failure modes

If this index drifts from the SSOT contract files, endpoint discovery and compatibility expectations become ambiguous.

## How to verify

```bash
make docs-lint-names
make api-contract-check
```

Expected output: index checks and API contract gates pass.

## See also

- [V1 Surface](V1_SURFACE.md)
- [Versioning](versioning.md)
- [Errors](errors.md)
