# API Versioning Strategy

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`

## Strategy

- URI-based versioning: `/v1/*`.
- OpenAPI document version is tracked in `ops/api/openapi-version-tracking.json`.
- Contract validation requires `info.version` to match tracked active version.
