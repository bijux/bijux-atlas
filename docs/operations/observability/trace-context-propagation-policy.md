# Trace Context Propagation Policy

- Propagate `traceparent` and `x-request-id` for all inbound and outbound request paths.
- Preserve context across async boundaries.
- Request and correlation identifiers must be mirrored in structured logs and response headers.
