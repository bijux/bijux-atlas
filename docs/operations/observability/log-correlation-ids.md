# Log Correlation IDs

Correlation chain:

- inbound `x-request-id`
- inbound `x-correlation-id`
- runtime trace context (`traceparent`)

Operator workflow:

1. Start with request id from alert or client report.
2. Pull correlated logs by `request_id`.
3. Expand by `correlation_id` for multi-request workflows.
4. Jump to traces with matching trace id.
