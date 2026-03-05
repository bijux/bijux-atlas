# Python Client API Reference

## `ClientConfig`

```python
ClientConfig(
    base_url: str,
    timeout_seconds: float = 10.0,
    max_retries: int = 2,
    backoff_seconds: float = 0.2,
    user_agent: str = "atlas-client/0.1.0",
    default_headers: dict[str, str] = {},
)
```

## `AtlasClient`

```python
AtlasClient(config: ClientConfig, logger: logging.Logger | None = None, trace_hook: TraceHook | None = None)
```

Methods:

- `query(request: QueryRequest) -> Page`
- `stream_query(request: QueryRequest) -> Iterator[dict[str, object]]`

## `QueryRequest`

```python
QueryRequest(
    dataset: str,
    filters: dict[str, Any] = {},
    fields: list[str] = [],
    limit: int | None = None,
    page_token: str | None = None,
)
```

## Errors

- `AtlasClientError`
- `AtlasConfigError`
- `AtlasApiError`
- `AtlasRetryExhaustedError`
