# Query Parameters

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: define canonical query parameters and required dimensions for dataset queries.

## Required dataset dimensions

- `release`
- `species`
- `assembly`

## Common optional query parameters

- `symbol`
- `gene_id`
- `chromosome`
- `start`
- `end`
- `biotype`
- `limit`
- `cursor`
- `fields`

## Rules

- Required dataset dimensions must always be provided.
- Unknown parameters must fail validation.
- Query parameter interpretation must remain stable across equivalent artifact versions.

## Related

- [Filtering](filtering.md)
- [Pagination](pagination.md)
- [Projections](projections.md)
