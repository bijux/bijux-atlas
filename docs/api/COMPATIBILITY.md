# API Compatibility

- Owner: `api`
- Stability: `stable`

## v1 Rules

- Contract source: `docs/contracts/ENDPOINTS.json`.
- API paths are frozen to `/v1/...`.
- v1 is additive-only:
- New endpoints are allowed.
- New optional params/fields are allowed.
- Removing or renaming endpoints/params/fields is forbidden.
- Tightening existing limits/defaults is forbidden unless version bump.

## Gates

- `make api-contract-check`
- `make openapi-drift`
