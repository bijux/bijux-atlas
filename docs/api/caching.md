# API Caching

## Contract

- ETag for dataset-backed read endpoints is computed from:
  - dataset artifact hash (`manifest.dataset_signature_sha256` when present, fallback dataset-id hash)
  - request path
  - normalized query string (sorted key/value pairs)
- `If-None-Match` returns `304 Not Modified` with no body.

## Cache-Control Policy

- Immutable dataset endpoints (`/v1/genes`, `/v1/sequence/*`, `/v1/diff/*`):
  - `Cache-Control: public, max-age=<ttl>, stale-while-revalidate=<ttl/2>, immutable`
- Catalog/discovery endpoints (`/v1/datasets`):
  - `Cache-Control: public, max-age=<ttl>, stale-while-revalidate=<ttl/2>`

## Vary

- All cached JSON/text responses set:
  - `Vary: accept-encoding`

## Debug Headers (Optional)

- When debug datasets mode is enabled, responses may include:
  - `X-Atlas-Artifact-Hash`
  - `X-Atlas-Cache-Key`

## CDN Notes

- CDN cache key should include full path + normalized query.
- Respect ETag revalidation and origin `Cache-Control`.
- Do not strip `Vary: accept-encoding`.
