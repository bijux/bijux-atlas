# PUBLIC API: bijux-atlas-api

Stability reference: [Stability Levels](../../../docs/_internal/style/stability-levels.md)

Stable public items for v1:

- `ApiErrorCode`
- `ApiError`
- `openapi_v1_spec()`
- `dataset_route_key()`
- `parse_list_genes_params()`
- `parse_list_genes_params_with_limit()`
- `parse_region_filter()`
- `ListGenesParams`
- `ContentNegotiation`
- `ApiResponseEnvelope<T>`
- `ApiContentType`

Stability rule:

- Additive changes only in v1.
- Existing enums may gain variants (`#[non_exhaustive]`).
