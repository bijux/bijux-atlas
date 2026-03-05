# bijux-atlas (Python SDK)

`bijux-atlas` is the Python SDK package for Bijux Atlas.

The SDK requires a compatible Bijux Atlas server runtime.

## Install

```bash
pip install bijux-atlas
```

## Import

```python
from bijux_atlas import AtlasClient, ClientConfig
```

## Quickstart

```python
from bijux_atlas import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig.from_env())
page = client.query(QueryRequest(dataset="genes", limit=5))
print(page.items)
```

Required environment variable:

- `BIJUX_ATLAS_URL`

Optional environment variables:

- `BIJUX_ATLAS_TOKEN`
- `BIJUX_ATLAS_TIMEOUT_SECONDS`
- `BIJUX_ATLAS_MAX_RETRIES`
- `BIJUX_ATLAS_BACKOFF_SECONDS`
- `BIJUX_ATLAS_MAX_BACKOFF_SECONDS`
- `BIJUX_ATLAS_VERIFY_SSL`
- `BIJUX_ATLAS_PROXY`
- `BIJUX_ATLAS_REQUEST_ID`

## Compatibility

Compatibility is declared in `compatibility.json` and checked by `AtlasClient.check_compatibility()`.
Runtime metadata is discovered from `/version` with fallback to `/health`.
Compatibility policy is based on server semantic version and API surface expectations derived from OpenAPI `v1` endpoints.

## Error Handling

Client failures raise typed exceptions from `bijux_atlas.errors`.
Server-side errors include `request_id` and `trace_id` when returned by the runtime.
Response schema shape validation is optional and disabled by default (`validate_response_schema=False`).

## Retries And Timeouts

Retries apply only to idempotent calls and use bounded linear backoff.
Timeouts and retry budgets are configurable via `ClientConfig`.
Streaming query traversal is supported through `AtlasClient.stream_query()`.

## Pagination

Paged responses are exposed via `Page` and `next_token`.
`AtlasClient.stream_query()` follows `next_page_token` until completion.

## Security And SSL

`ClientConfig` supports TLS verification control (`verify_ssl`) and proxy routing (`proxy_url`).
Auth tokens are sent via `Authorization: Bearer <token>`.
Request correlation is supported through `BIJUX_ATLAS_REQUEST_ID` / `request_id`.

## Telemetry

Telemetry is optional and disabled by default.
You can pass a logger and/or trace hook into `AtlasClient`.

## Packaging scope

- SDK source: `src/bijux_atlas/`
- Examples: `examples/`
- Tests: `tests/`
- Docs: `docs/`
- Notebooks: `notebooks/`
