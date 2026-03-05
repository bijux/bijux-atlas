# Python Client Troubleshooting

## Connection refused

- Verify runtime is available at `base_url`.
- Check local port mapping and firewall.

## Retry exhausted

- Increase `timeout_seconds` and `max_retries`.
- Inspect network stability and runtime logs.

## HTTP 4xx responses

- Validate dataset name and query filters.
- Ensure endpoint path is `POST /v1/query`.
