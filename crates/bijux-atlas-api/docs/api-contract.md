# API Contract (v1)

This crate defines Atlas wire contracts only.

## Stability

- Request parsing and validation are deterministic.
- Error schema is stable and backed by SSOT error codes.
- OpenAPI paths must match SSOT endpoint registry.

## Surface

- Params parsing (`parse_list_genes_params*`, `parse_region_filter`).
- Error envelope (`ApiError`).
- Response envelope/content-negotiation helpers.
- OpenAPI generation (`openapi_v1_spec`).

## Non-goals

- No runtime I/O, no DB access, no store access.
- No server orchestration logic.
