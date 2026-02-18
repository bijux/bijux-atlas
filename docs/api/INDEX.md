# API Surface Index

- Owner: `api`
- Stability: `stable`

Purpose: single entrypoint for v1 API surface, dataset selection rules, pagination, filters, and error model.

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
